package io.github.artcraftandroid.filmcraft.bridge

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.FloatingActionButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

@Composable
fun AiImportOverlay(
    open: Boolean, script: String, reviewed: AiScript?, busy: Boolean, status: String,
    onOpen: () -> Unit, onClose: () -> Unit, onChange: (String) -> Unit,
    onReview: () -> Unit, onApply: () -> Unit
) {
    Box(Modifier.fillMaxSize()) {
        FloatingActionButton(
            onClick = onOpen,
            modifier = Modifier.align(Alignment.BottomStart).padding(start = 18.dp, bottom = 24.dp)
        ) { Text("AI") }
    }
    if (!open) return
    AlertDialog(
        onDismissRequest = { if (!busy) onClose() },
        title = { Text("FilmCraft AI Script") },
        text = {
            Column(Modifier.heightIn(max = 460.dp).verticalScroll(rememberScrollState()),
                verticalArrangement = Arrangement.spacedBy(12.dp)) {
                Text("Paste a JSON script created in ChatGPT. Review, then tap Apply. Nothing runs automatically.")
                OutlinedTextField(
                    value = script, onValueChange = onChange,
                    label = { Text("Editing script JSON") },
                    modifier = Modifier.fillMaxWidth(),
                    minLines = 5, maxLines = 10, enabled = !busy
                )
                if (reviewed != null) {
                    Text("Review ${reviewed.steps.size} actions:")
                    reviewed.steps.forEachIndexed { i, step -> Text("${i + 1}. ${step.label}") }
                }
                if (status.isNotBlank()) Text(status)
            }
        },
        confirmButton = {
            Row(horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                TextButton(onClick = onReview, enabled = !busy) { Text("Review") }
                Button(onClick = onApply, enabled = !busy && reviewed != null) { Text("Apply") }
            }
        },
        dismissButton = { TextButton(onClick = onClose, enabled = !busy) { Text("Close") } }
    )
}
