//! Speed changes that keep the pitch (Speed/Duration ▸ Maintain Audio Pitch): the clip's sound is
//! time-stretched by WSOLA (waveform-similarity overlap-add) instead of resampled.
//!
//! The sound is cut into 40 ms grains, half overlapping under a Hann window. Each grain is copied
//! from the source at the source's own rate, so the pitch stays, from where the clip's timing puts
//! its centre, nudged by up to 10 ms so that it lines up with the grain before it. A grain's place
//! is fixed by its index on the clip's own sample grid, and the nudges start again every
//! [`BLOCK`] grains, so a span of the clip comes out the same however the timeline is cut into
//! blocks: playback, scrubbing and export agree sample for sample.

use filmcraft_frame::AudioBuffer;

/// Grain length.
const GRAIN_SECONDS: f64 = 0.040;
/// How far a grain may be nudged to line up with the one before it.
const NUDGE_SECONDS: f64 = 0.010;
/// The nudges chain within blocks of this many grains.
const BLOCK: i64 = 8;
/// The search tries every 4th nudge, then the ones around the best.
const COARSE: i64 = 4;
/// The similarity of two stretches is measured on every 4th sample.
const STRIDE: usize = 4;
/// The most media one call reads (10 minutes at 48 kHz): a guard against absurd requests.
const MAX_SPAN: i64 = 48_000 * 600;

/// `len` samples of a clip played at `speed` with its pitch kept, starting `j0` samples after the
/// clip's start, at `sr`. The clip starts on media sample `m0` (fractional); `read(start, n)` gives
/// `n` media samples from media sample `start`. `None` when the speed is unusable or nothing could
/// be read.
pub fn stretch(read: &dyn Fn(i64, usize) -> Option<AudioBuffer>, m0: f64, speed: f64, j0: i64, len: usize, sr: u32) -> Option<AudioBuffer> {
    // (positions past about 700 years of sound are not a clip's)
    const FAR: f64 = (1u64 << 50) as f64;
    if !(speed.is_finite() && speed > 1e-6 && speed < 1e3 && m0.is_finite() && m0.abs() < FAR) || sr == 0 || (j0 as f64).abs() >= FAR || len as f64 >= FAR {
        return None;
    }
    let grain = ((f64::from(sr) * GRAIN_SECONDS) as i64 / 2 * 2).max(16);
    let hop = grain / 2;
    let nudge = ((f64::from(sr) * NUDGE_SECONDS) as i64).max(1);
    // the media sample grain `k` starts on before its nudge: its centre (timeline sample
    // k·hop + hop) lands where the clip's timing puts it
    let nominal = |k: i64| -> i64 { (m0 + ((k.saturating_mul(hop) + hop) as f64) * speed).floor() as i64 - hop };
    let len_i = i64::try_from(len).ok()?;
    let first = (j0 - grain).div_euclid(hop) + 1;
    let last = (j0 + len_i - 1).div_euclid(hop);
    let chain = first.div_euclid(BLOCK) * BLOCK;
    let lo = nominal(chain) - nudge;
    let hi = nominal(last) + nudge + grain;
    let span = hi - lo;
    if span <= 0 || span > MAX_SPAN {
        return None;
    }
    // before the start of the media there is silence
    let from = lo.max(0);
    let raw = if hi > from { read(from, usize::try_from(hi - from).ok()?)? } else { AudioBuffer::silence(sr, 2, 0) };
    let channels = raw.channels.len().max(1);
    let at = |c: usize, i: i64| -> f32 {
        if i < from {
            return 0.0;
        }
        usize::try_from(i - from).ok().and_then(|o| raw.channels.get(c).and_then(|ch| ch.get(o))).copied().unwrap_or(0.0)
    };
    // a mono guide for the search
    let guide: Vec<f32> = (0..usize::try_from(hi - from).unwrap_or(0))
        .map(|o| {
            let i = from + o as i64;
            (0..channels).map(|c| at(c, i)).sum::<f32>() / channels as f32
        })
        .collect();
    let g = |i: i64| -> f32 { if i < from { 0.0 } else { usize::try_from(i - from).ok().and_then(|o| guide.get(o)).copied().unwrap_or(0.0) } };
    let similarity = |a: i64, b: i64| -> f32 { (0..hop).step_by(STRIDE).map(|t| g(a + t) * g(b + t)).sum() };
    // each grain's media start, nudged to continue the one before it
    let mut starts: Vec<i64> = Vec::with_capacity(usize::try_from(last - chain + 1).unwrap_or(0));
    for k in chain..=last {
        let base = nominal(k);
        let prev = if k.rem_euclid(BLOCK) == 0 { None } else { starts.last().copied() };
        let start = match prev {
            None => base,
            Some(p) => {
                // the grain before it, carried on past its middle
                let want = p + hop;
                let mut best = (f32::MIN, 0);
                let mut d = -nudge;
                while d <= nudge {
                    let s = similarity(want, base + d);
                    if s > best.0 {
                        best = (s, d);
                    }
                    d += COARSE;
                }
                let around = best.1;
                for d in (around - COARSE + 1)..(around + COARSE) {
                    if d < -nudge || d > nudge || d == around {
                        continue;
                    }
                    let s = similarity(want, base + d);
                    if s > best.0 {
                        best = (s, d);
                    }
                }
                base + best.1
            }
        };
        starts.push(start);
    }
    // overlap-add under a periodic Hann window (half-overlapping windows sum to one)
    let window: Vec<f32> = (0..grain).map(|t| (0.5 - 0.5 * (std::f64::consts::TAU * t as f64 / grain as f64).cos()) as f32).collect();
    let mut out = AudioBuffer::silence(sr, channels, len);
    for (idx, k) in (chain..=last).enumerate() {
        if k < first {
            continue;
        }
        let Some(&start) = starts.get(idx) else { continue };
        let g0 = k * hop;
        let t0 = (j0 - g0).max(0);
        let t1 = (j0 + len_i - g0).min(grain);
        for (c, ch) in out.channels.iter_mut().enumerate() {
            for t in t0..t1 {
                let w = window.get(usize::try_from(t).unwrap_or(0)).copied().unwrap_or(0.0);
                if let Some(o) = usize::try_from(g0 + t - j0).ok().and_then(|o| ch.get_mut(o)) {
                    *o += w * at(c, start + t);
                }
            }
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SR: u32 = 48_000;

    fn sine(hz: f64) -> impl Fn(i64, usize) -> Option<AudioBuffer> {
        move |start, n| {
            let ch: Vec<f32> = (0..n).map(|i| ((std::f64::consts::TAU * hz * (start + i as i64) as f64 / f64::from(SR)).sin() * 0.5) as f32).collect();
            Some(AudioBuffer { sample_rate: SR, channels: vec![ch.clone(), ch] })
        }
    }

    /// The frequency of a stretch, from its rising zero crossings.
    fn pitch(x: &[f32]) -> f64 {
        let ups: Vec<usize> = x.windows(2).enumerate().filter(|(_, w)| w[0] < 0.0 && w[1] >= 0.0).map(|(i, _)| i).collect();
        let (Some(a), Some(b)) = (ups.first(), ups.last()) else { return 0.0 };
        (ups.len() - 1) as f64 * f64::from(SR) / (b - a) as f64
    }

    #[test]
    fn the_pitch_stays_at_any_speed() {
        for speed in [0.5, 0.8, 1.5, 2.0, 4.0] {
            let out = stretch(&sine(440.0), 0.0, speed, 4_800, 24_000, SR).unwrap();
            let p = pitch(&out.channels[0]);
            assert!((p - 440.0).abs() < 6.0, "{speed}x: {p} Hz");
            let level = out.channels[0].iter().map(|s| s.abs()).fold(0.0, f32::max);
            assert!((0.4..=0.6).contains(&level), "{speed}x: peak {level}");
        }
    }

    #[test]
    fn any_cut_of_the_timeline_gives_the_same_samples() {
        let read = sine(310.0);
        let whole = stretch(&read, 1234.5, 1.7, 1_000, 9_000, SR).unwrap();
        let mut parts = Vec::new();
        for (j0, n) in [(1_000, 1_024), (2_024, 3_000), (5_024, 4_976)] {
            parts.extend(stretch(&read, 1234.5, 1.7, j0, n, SR).unwrap().channels[0].clone());
        }
        assert_eq!(parts, whole.channels[0]);
    }

    #[test]
    fn it_follows_the_clips_timing() {
        // a click every second of media: at twice the speed, every half second of the clip
        let read = |start: i64, n: usize| -> Option<AudioBuffer> {
            let ch: Vec<f32> = (0..n).map(|i| if (start + i as i64).rem_euclid(48_000) < 48 { 1.0 } else { 0.0 }).collect();
            Some(AudioBuffer { sample_rate: SR, channels: vec![ch] })
        };
        let out = stretch(&read, 0.0, 2.0, 0, 96_000, SR).unwrap();
        let loud: Vec<usize> = out.channels[0].iter().enumerate().filter(|(_, s)| s.abs() > 0.3).map(|(i, _)| i).collect();
        for want in [24_000usize, 48_000, 72_000] {
            assert!(loud.iter().any(|&i| i.abs_diff(want) < 600), "a click near {want}: {:?}", &loud[..loud.len().min(8)]);
        }
    }

    #[test]
    fn bad_input_is_none_not_a_crash() {
        let read = sine(440.0);
        assert!(stretch(&read, 0.0, 0.0, 0, 100, SR).is_none());
        assert!(stretch(&read, 0.0, f64::NAN, 0, 100, SR).is_none());
        assert!(stretch(&read, f64::INFINITY, 1.0, 0, 100, SR).is_none());
        assert!(stretch(&read, 0.0, 2.0, i64::MAX / 4, 100, SR).is_none());
        assert!(stretch(&read, 0.0, 2.0, 0, usize::MAX, SR).is_none());
        assert!(stretch(&read, 0.0, 2.0, 0, 100, 0).is_none());
        assert_eq!(stretch(&read, -50_000.0, 1.5, 0, 100, SR).unwrap().channels[0].len(), 100);
    }
}
