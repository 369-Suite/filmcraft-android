package io.github.artcraftandroid.filmcraft.bridge

import io.github.artcraftandroid.filmcraft.ui.AppViewModel
import org.json.JSONObject

/** Offline, reviewed command set. No API key or network listener. */
data class AiStep(val op: String, val args: JSONObject, val label: String)
data class AiScript(val steps: List<AiStep>)

object AiScriptParser {
    private val operations = setOf("project.create_blank", "editor.add_blank", "editor.add_text")
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
