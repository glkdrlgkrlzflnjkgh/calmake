# Calmake for VS Code

This extension adds `build.cal` syntax highlighting, lightweight editor diagnostics,
and commands for the calmake build workflow.

Install the extension from this directory while developing:

```text
code --extensionDevelopmentPath=./calmake-vscode
```

Set `calmake.executable` when `calmake` is not available on `PATH`. The extension
automatically regenerates `compile_commands.json` when `build.cal` is saved, unless
`calmake.autoGenerateCompileCommands` is disabled.