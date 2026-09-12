const vscode = require('vscode');
const path = require('path');

function workspaceRoot() {
  return vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
}

function executable() {
  return vscode.workspace.getConfiguration('calmake').get('executable', 'calmake');
}

function calmakeTask(command) {
  const root = workspaceRoot();
  if (!root) return undefined;

  const definition = { type: 'calmake', command };
  const execution = new vscode.ShellExecution(executable(), [command]);
  const task = new vscode.Task(
    definition,
    vscode.TaskScope.Workspace,
    `Calmake: ${command}`,
    'calmake',
    execution,
  );
  task.presentationOptions = {
    reveal: vscode.TaskRevealKind.Always,
    panel: vscode.TaskPanelKind.Shared,
    clear: false,
    close: false,
    focus: false,
    showReuseMessage: true,
  };
  task.detail = `Run calmake ${command} in the integrated terminal`;
  return task;
}

function executeCalmake(command) {
  const task = calmakeTask(command);
  if (!task) return;
  vscode.tasks.executeTask(task);
}

function isCalmakeTask(task, command) {
  return task.definition?.type === 'calmake' && task.definition.command === command;
}

function diagnosticsFor(document, collection) {
  if (document.languageId !== 'calmake') return;
  const diagnostics = [];
  const lines = document.getText().split(/\r?\n/);
  let target = undefined;
  const knownProperties = new Set(['kind', 'language', 'sources', 'deps', 'output', 'cflags', 'cppflags', 'ldflags']);
  const targetNames = new Set();
  const addDiagnostic = (index, text, message, severity = vscode.DiagnosticSeverity.Error, start = 0) => {
    diagnostics.push(new vscode.Diagnostic(
      new vscode.Range(index, start, index, Math.max(start + 1, text.length)),
      message,
      severity,
    ));
  };
  const closeTarget = (index, text) => {
    if (!target) return;
    if (!target.properties.has('output')) addDiagnostic(index, text, `Target '${target.name}' is missing 'output'.`);
    if (!target.properties.has('sources')) addDiagnostic(index, text, `Target '${target.name}' is missing 'sources'.`);
    target = undefined;
  };

  lines.forEach((text, index) => {
    const line = text.trim();
    if (!line || line.startsWith('#')) return;

    if (line.startsWith('target ')) {
      if (target) {
        addDiagnostic(index, text, 'Nested target declarations are not allowed.');
        return;
      }
      if (!line.endsWith('{')) {
        addDiagnostic(index, text, 'Target declaration must end with `{`.');
        return;
      }
      const name = line.slice('target '.length, -1).trim();
      if (!name) {
        addDiagnostic(index, text, 'Target name is required.');
        return;
      }
      if (targetNames.has(name)) addDiagnostic(index, text, `Duplicate target '${name}'.`);
      targetNames.add(name);
      target = { name, properties: new Set() };
      return;
    }

    if (line === '}') {
      if (!target) addDiagnostic(index, text, 'Unexpected closing brace.');
      else closeTarget(index, text);
      return;
    }

    if (!target) {
      addDiagnostic(index, text, 'Property is outside of a target block.');
      return;
    }

    const equals = line.indexOf('=');
    if (equals < 0) {
      addDiagnostic(index, text, 'Expected `key = value`.');
      return;
    }
    const key = line.slice(0, equals).trim();
    const value = line.slice(equals + 1).trim();
    if (!key) {
      addDiagnostic(index, text, 'Property name is required.');
      return;
    }
    if (!value) {
      addDiagnostic(index, text, `Property '${key}' requires a value.`);
      return;
    }
    if (!knownProperties.has(key)) {
      addDiagnostic(index, text, `Unknown property '${key}'.`);
      return;
    }
    if (target.properties.has(key)) {
      addDiagnostic(index, text, `Property '${key}' is duplicated in target '${target.name}'.`);
    }
    target.properties.add(key);

    if (key === 'kind' && !['exe', 'staticlib', 'sharedlib'].includes(value.replace(/^"|"$/g, ''))) {
      addDiagnostic(index, text, `Invalid kind '${value}'. Expected exe, staticlib, or sharedlib.`);
    }
    if (key === 'language' && !['c', 'cpp'].includes(value.replace(/^"|"$/g, ''))) {
      addDiagnostic(index, text, `Invalid language '${value}'. Expected c or cpp.`);
    }
    if (value.startsWith('"') !== value.endsWith('"')) {
      addDiagnostic(index, text, 'Unterminated quoted value.');
    }
    if (value.startsWith('[') !== value.endsWith(']')) {
      addDiagnostic(index, text, 'List values must be enclosed in `[` and `]`.');
    }
  });

  if (target) addDiagnostic(lines.length - 1, lines[lines.length - 1], `Unterminated target block '${target.name}'.`);
  collection.set(document.uri, diagnostics);
}

class CalmakeTaskProvider {
  provideTasks() {
    return ['build', 'check', 'compdb', 'graph', 'clean']
      .map(command => calmakeTask(command))
      .filter(Boolean);
  }

  resolveTask(task) {
    const command = task.definition?.command;
    if (!['build', 'check', 'compdb', 'graph', 'clean'].includes(command)) return undefined;
    return calmakeTask(command);
  }
}

function activate(context) {
  const diagnostics = vscode.languages.createDiagnosticCollection('calmake');
  let compdbState = 'idle';
  let compdbQueued = false;

  const executeCompdb = () => {
    if (compdbState !== 'idle') {
      compdbQueued = true;
      return;
    }

    const task = calmakeTask('compdb');
    if (!task) return;
    compdbState = 'launching';
    vscode.tasks.executeTask(task);
  };

  context.subscriptions.push(diagnostics);
  context.subscriptions.push(vscode.tasks.registerTaskProvider('calmake', new CalmakeTaskProvider()));
  context.subscriptions.push(vscode.tasks.onDidStartTask(event => {
    if (isCalmakeTask(event.execution.task, 'compdb')) {
      compdbState = 'running';
    }
  }));
  context.subscriptions.push(vscode.tasks.onDidEndTask(event => {
    if (!isCalmakeTask(event.execution.task, 'compdb')) return;
    compdbState = 'idle';
    if (compdbQueued) {
      compdbQueued = false;
      executeCompdb();
    }
  }));
  context.subscriptions.push(vscode.commands.registerCommand('calmake.build', () => executeCalmake('build')));
  context.subscriptions.push(vscode.commands.registerCommand('calmake.check', () => executeCalmake('check')));
  context.subscriptions.push(vscode.commands.registerCommand('calmake.generateCompileCommands', executeCompdb));
  context.subscriptions.push(vscode.commands.registerCommand('calmake.graph', () => executeCalmake('graph')));
  context.subscriptions.push(vscode.commands.registerCommand('calmake.clean', () => executeCalmake('clean')));
  context.subscriptions.push(vscode.workspace.onDidOpenTextDocument(document => diagnosticsFor(document, diagnostics)));
  context.subscriptions.push(vscode.workspace.onDidChangeTextDocument(event => diagnosticsFor(event.document, diagnostics)));
  context.subscriptions.push(vscode.workspace.onDidSaveTextDocument(document => {
    diagnosticsFor(document, diagnostics);
    if (document.fileName.endsWith(path.join('', 'build.cal')) && vscode.workspace.getConfiguration('calmake').get('autoGenerateCompileCommands', true)) {
      executeCompdb();
    }
  }));
  vscode.workspace.textDocuments.forEach(document => diagnosticsFor(document, diagnostics));
}

function deactivate() {}

module.exports = { activate, deactivate };