package io.github.artcraftandroid.filmcraft.bridge

import io.github.artcraftandroid.filmcraft.ui.AppViewModel
import io.github.artcraftandroid.filmcraft.ui.editor.TPS
import org.json.JSONObject

/** Offline, reviewed command set. No API key or network listener. */
data class AiStep(val op: String, val args: JSONObject, val label: String)
data class AiScript(val steps: List<AiStep>)

object AiScriptParser {
    private val operations = setOf("project.create_blank", "project.create_from_media", "editor.add_blank", "editor.add_text", "editor.seek_seconds", "editor.open_export")
    fun parse(input: String): AiScript {
        require(input.length <= 10_000) { "Script is too long" }
        var src = input.trim()
        if (src.startsWith("```")) {
            val newline = src.indexOf('\n')
            require(newline >= 0 && src.endsWith("```")) { "Incomplete JSON code block" }
            src = src.substring(newline + 1, src.length - 3).trim()
        }
        val obj = JSONObject(src)
        require(obj.optInt("version", -1) == 1) { "Expected version 1" }
        val commands = obj.optJSONArray("commands") ?: error("Missing commands array")
        require(commands.length() in 1..10) { "Use 1 to 10 commands" }
        return AiScript((0 until commands.length()).map { i ->
            val cmd = commands.getJSONObject(i)
            val op = cmd.getString("op")
            require(op in operations) { "Unsupported operation: $op" }
            val args = cmd.optJSONObject("args") ?: JSONObject()
            val label = when (op) {
                "project.create_blank" -> {
                    require(i == 0) { "Create project must come first" }
                    val name = args.opt("name")
                    require(name is String && name.isNotBlank() && name.length <= 80) { "Project name must be 1-80 characters" }
                    "Create project: $name"
                }
                "project.create_from_media" -> {
                    require(i == 0) { "Import video must be the first command" }
                    val name = args.opt("name")
                    require(name is String && name.isNotBlank() && name.length <= 80) { "Project name must be 1-80 characters" }
                    "Ask you to choose a video, then create: $name"
                }
                "editor.seek_seconds" -> {
                    val at = args.optDouble("seconds", Double.NaN)
                    require(at.isFinite() && at in 0.0..7200.0) { "Position must be 0 to 7200 seconds" }
                    "Move playhead to ${"%.2f".format(java.util.Locale.US, at)} seconds"
                }
                "editor.open_export" -> {
                    require(i == commands.length() - 1) { "Open export must be last" }
                    "Open export settings for your approval"
                }
                "editor.add_blank" -> "Add a blank clip"
                "editor.add_text" -> {
                    val value = args.opt("text")
                    require(value is String && value.isNotBlank() && value.length <= 1000) { "Text must be 1-1000 characters" }
                    "Add text: ${value.take(55)}"
                }
                else -> error("Unsupported")
            }
            AiStep(op, args, label)
        })
    }
}

object AiLocalExecutor {
    suspend fun execute(vm: AppViewModel, step: AiStep) {
        when (step.op) {
            "project.create_blank" -> vm.createBlankForBridge(step.args.getString("name"))
            "project.create_from_media" -> vm.createFromMediaForBridge(step.args.getString("name"))
            "editor.open_export" -> {
                check(vm.editor != null) { "Open a project first" }
                vm.export()
            }
            "editor.seek_seconds" -> {
                val ed = vm.editor ?: error("Open a project first")
                val requested = (step.args.getDouble("seconds") * TPS).toLong()
                val tick = requested.coerceIn(0L, ed.duration)
                ed.stop()
                ed.engine.command { setPlayhead(tick) }
                ed.refresh()
            }
            "editor.add_blank", "editor.add_text" -> {
                val ed = vm.editor ?: error("Open a project first")
                ed.stop()
                if (step.op == "editor.add_blank") ed.engine.command { addBlank() }
                else ed.engine.command { addText(step.args.getString("text")) }
                ed.refresh()
                ed.save()
            }
            else -> error("Command rejected")
        }
    }
}
