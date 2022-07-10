import { describe, expect, it } from 'vitest'
import { XBASE_BIN_ROOT } from '../src/constants'
import XBaseServer from '../src/server'

describe('XBaseServer', async () => {
  it('Should connect with a support project root', async () => {
    // NOTE: XBaseServer.connect() might auto spawn xbase if it isn't already running
    expect(XBASE_BIN_ROOT).toBe(`/${process.env.HOME}/.local/share/xbase`)
    const connect = await XBaseServer.connect()

    expect(connect.isOk(), 'XBaseServer initilaized without errors').toBe(true)

    const server = connect.unwrap()
    const register = await server.register(`/${process.env.TESTPROJECT}`)

    expect(register.isOk(), 'Register request is sent without errors').toBe(true)

    const address = register.unwrap()
    expect(typeof address).toBe('string')
  })
})
