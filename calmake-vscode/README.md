# Calmake for VS Code

This extension adds `build.cal` syntax highlighting, lightweight editor diagnostics,
and terminal-backed tasks for the calmake build workflow.

The following tasks are available from **Terminal: Run Task**:

- `Calmake: build`
- `Calmake: compdb`
- `Calmake: graph`
- `Calmake: clean`

The Calmake commands in the Command Palette execute the same tasks. Calmake stdout
and stderr stay in the integrated Terminal instead of being converted into pop-up
notifications.

Install the extension from this directory while developing:

```text
code --extensionDevelopmentPath=./calmake-vscode
```

Set `calmake.executable` when `calmake` is not available on `PATH`. The extension
automatically regenerates `compile_commands.json` when `build.cal` is saved, unless
`calmake.autoGenerateCompileCommands` is disabled.