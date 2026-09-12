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

function diagnosticsFor(document, collection) {
  if (document.languageId !== 'calmake') return;
  const diagnostics = [];
  const lines = document.getText().split(/\r?\n/);
  let targetDepth = 0;
  lines.forEach((text, index) => {
    const line = text.trim();
    if (!line || line.startsWith('#')) return;
    if (line.startsWith('target ') && line.endsWith('{')) {
      targetDepth += 1;
      return;
    }
    if (line === '}') {
      if (targetDepth === 0) {
        diagnostics.push(new vscode.Diagnostic(new vscode.Range(index, 0, index, text.length), 'Unexpected closing brace.', vscode.DiagnosticSeverity.Error));
      } else {
        targetDepth -= 1;
      }
      return;
    }
    if (targetDepth === 0) {
      diagnostics.push(new vscode.Diagnostic(new vscode.Range(index, 0, index, text.length), 'Properties must be inside a target block.', vscode.DiagnosticSeverity.Error));
      return;
    }
    if (!line.includes('=')) {
      diagnostics.push(new vscode.Diagnostic(new vscode.Range(index, 0, index, text.length), 'Expected `key = value`.', vscode.DiagnosticSeverity.Error));
    }
  });
  if (targetDepth > 0) {
    const lastLine = Math.max(0, lines.length - 1);
    diagnostics.push(new vscode.Diagnostic(new vscode.Range(lastLine, 0, lastLine, lines[lastLine].length), 'Unterminated target block.', vscode.DiagnosticSeverity.Error));
  }
  collection.set(document.uri, diagnostics);
}

class CalmakeTaskProvider {
  provideTasks() {
    return ['build', 'compdb', 'graph', 'clean']
      .map(command => calmakeTask(command))
      .filter(Boolean);
  }

  resolveTask(task) {
    const command = task.definition?.command;
    if (!['build', 'compdb', 'graph', 'clean'].includes(command)) return undefined;
    return calmakeTask(command);
  }
}

function activate(context) {
  const diagnostics = vscode.languages.createDiagnosticCollection('calmake');
  context.subscriptions.push(diagnostics);
  context.subscriptions.push(vscode.tasks.registerTaskProvider('calmake', new CalmakeTaskProvider()));
  context.subscriptions.push(vscode.commands.registerCommand('calmake.build', () => executeCalmake('build')));
  context.subscriptions.push(vscode.commands.registerCommand('calmake.generateCompileCommands', () => executeCalmake('compdb')));
  context.subscriptions.push(vscode.commands.registerCommand('calmake.graph', () => executeCalmake('graph')));
  context.subscriptions.push(vscode.commands.registerCommand('calmake.clean', () => executeCalmake('clean')));
  context.subscriptions.push(vscode.workspace.onDidOpenTextDocument(document => diagnosticsFor(document, diagnostics)));
  context.subscriptions.push(vscode.workspace.onDidChangeTextDocument(event => diagnosticsFor(event.document, diagnostics)));
  context.subscriptions.push(vscode.workspace.onDidSaveTextDocument(document => {
    diagnosticsFor(document, diagnostics);
    if (document.fileName.endsWith(path.join('', 'build.cal')) && vscode.workspace.getConfiguration('calmake').get('autoGenerateCompileCommands', true)) {
      executeCalmake('compdb');
    }
  }));
  vscode.workspace.textDocuments.forEach(document => diagnosticsFor(document, diagnostics));
}

function deactivate() {}

module.exports = { activate, deactivate };