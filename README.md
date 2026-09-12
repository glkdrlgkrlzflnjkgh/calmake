calmake is a build system.

Commands:

- `calmake build` builds the current `build.cal` project and refreshes `compile_commands.json`.
- `calmake check` validates `build.cal` and reports all recoverable syntax errors.
- `calmake compdb` only regenerates `compile_commands.json` for clangd and other tooling.
- `calmake graph` prints the target graph as Graphviz DOT.
- `calmake init <directory>` creates a sample project.

The compilation database uses `arguments` entries and the same compiler flags as calmake's
compile steps, so clangd can be pointed at the project root without a separate configuration.

Parser errors include line and column locations and are aggregated into one report, so a
single `calmake check` run can identify several independent problems at once.
