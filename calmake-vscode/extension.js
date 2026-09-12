const vscode = require('vscode');
const cp = require('child_process');
const path = require('path');

function workspaceRoot() {
  return vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
}

function executable() {
  return vscode.workspace.getConfiguration('calmake').get('executable', 'calmake');
}

function runCalmake(args) {
  const root = workspaceRoot();
  if (!root) {
    return Promise.reject(new Error('Open a calmake project folder first.'));
  }

  return new Promise((resolve, reject) => {
    const child = cp.spawn(executable(), args, { cwd: root, shell: process.platform === 'win32' });
    let output = '';
    child.stdout.on('data', data => { output += data; });
    child.stderr.on('data', data => { output += data; });
    child.on('error', reject);
    child.on('close', code => {
      if (code === 0) resolve(output);
      else reject(new Error(output.trim() || `calmake exited with code ${code}`));
    });
  });
}

async function runCommand(args, successMessage) {
  try {
    const output = await runCalmake(args);
    if (output.trim()) {
      vscode.window.showInformationMessage(output.trim().split(/\r?\n/).pop());
    } else if (successMessage) {
      vscode.window.showInformationMessage(successMessage);
    }
  } catch (error) {
    vscode.window.showErrorMessage(`Calmake: ${error.message}`);
  }
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

function activate(context) {
  const diagnostics = vscode.languages.createDiagnosticCollection('calmake');
  context.subscriptions.push(diagnostics);
  context.subscriptions.push(vscode.commands.registerCommand('calmake.build', () => runCommand(['build'], 'Build complete.')));
  context.subscriptions.push(vscode.commands.registerCommand('calmake.generateCompileCommands', () => runCommand(['compdb'], 'compile_commands.json generated.')));
  context.subscriptions.push(vscode.commands.registerCommand('calmake.graph', async () => {
    try {
      const output = await runCalmake(['graph']);
      const document = await vscode.workspace.openTextDocument({ content: output, language: 'plaintext' });
      await vscode.window.showTextDocument(document, { preview: true });
    } catch (error) {
      vscode.window.showErrorMessage(`Calmake: ${error.message}`);
    }
  }));
  context.subscriptions.push(vscode.commands.registerCommand('calmake.clean', () => runCommand(['clean'], 'Clean complete.')));
  context.subscriptions.push(vscode.workspace.onDidOpenTextDocument(document => diagnosticsFor(document, diagnostics)));
  context.subscriptions.push(vscode.workspace.onDidChangeTextDocument(event => diagnosticsFor(event.document, diagnostics)));
  context.subscriptions.push(vscode.workspace.onDidSaveTextDocument(document => {
    diagnosticsFor(document, diagnostics);
    if (document.fileName.endsWith(path.join('', 'build.cal')) && vscode.workspace.getConfiguration('calmake').get('autoGenerateCompileCommands', true)) {
      runCommand(['compdb']);
    }
  }));
  vscode.workspace.textDocuments.forEach(document => diagnosticsFor(document, diagnostics));
}

function deactivate() {}

module.exports = { activate, deactivate };