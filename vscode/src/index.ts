import type { ExtensionContext } from 'vscode'
import { commands, window, workspace } from 'vscode'
import * as util from './util'
import XBaseServer from './server'

let server: XBaseServer

export async function activate(_context: ExtensionContext) {
  console.debug('Activating XBase')
  const connect = await XBaseServer.connect()

  let root: string

  if (workspace.workspaceFolders && (workspace.workspaceFolders.length > 0)) {
    root = workspace.workspaceFolders[0].uri.fsPath
  }
  else if (workspace.workspaceFolders) {
    root = workspace.workspaceFolders[0].uri.fsPath
    console.warn(`Found multiple roots, using ${root}`)
  }
  else {
    console.error('No Workspace Opened, ignoring ..')
    return
  }

  if (connect.isErr()) {
    console.error(`Fail to activate XBase: ${connect.unwrapErr()}`)
    return
  }
  else { server = connect.unwrap() }

  // TODO: Setup Editor Status
  const register = await server.register(root)
  if (register.isErr()) {
    console.error(`Fail to register workspaceFolder: ${register.unwrapErr()}`)
    return
  }
  else {
    const name = util.nameFromPath(root)
    window.showInformationMessage(`[${name}] Registered`)
  }

  // TODO: Move commands to commands.ts
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

export function deactivate(_context: ExtensionContext) {
  // TODO: Drop all roots
  server.socket.end()
}
