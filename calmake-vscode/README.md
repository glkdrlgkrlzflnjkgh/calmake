# Calmake for VS Code

This extension adds `build.cal` syntax highlighting, lightweight editor diagnostics,
and terminal-backed tasks for the calmake build workflow.

The editor reports multiple independent syntax problems at once, including malformed
target declarations, duplicate targets, missing required properties, unknown properties,
invalid enum values, malformed values, stray braces, and unterminated target blocks.

The following tasks are available from **Terminal: Run Task**:

- `Calmake: build`
- `Calmake: check`
- `Calmake: compdb`
- `Calmake: graph`
- `Calmake: clean`

The Calmake commands in the Command Palette execute the same tasks. Calmake stdout
and stderr stay in the integrated Terminal instead of being converted into pop-up
notifications. Compile database requests are serialized: if `build.cal` is saved
while `compdb` is running, one follow-up `compdb` runs after the current task ends.

Install the extension from this directory while developing:

```text
code --extensionDevelopmentPath=./calmake-vscode
```

Set `calmake.executable` when `calmake` is not available on `PATH`. The extension
automatically regenerates `compile_commands.json` when `build.cal` is saved, unless
`calmake.autoGenerateCompileCommands` is disabled.