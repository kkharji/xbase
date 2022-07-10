import { commands, window } from 'vscode'

export function activate() {
  commands.registerCommand('xbase.run', () => {
    window.showInformationMessage('Pick target/scheme to run ...')
  })
  commands.registerCommand('xbase.build', () => {
    window.showInformationMessage('Pick target/scheme to build ...')
  })
  commands.registerCommand('xbase.watch', () => {
    window.showInformationMessage('Pick ...')
  })
}

export function deactivate() {
  // TODO: Drop all roots
  // TODO: Close connection

}
