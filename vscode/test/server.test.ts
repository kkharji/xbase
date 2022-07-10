import { describe, expect, it } from "vitest";
import XBaseServer from "../src/server";

describe("XBaseServer", async () => {
  it("Should connect with a support project root", async () => {
    // NOTE: XBaseServer.connect() might auto spawn xbase if it isn't already running

    let connect = await XBaseServer.connect();

    expect(connect.isOk(), "XBaseServer initilaized without errors").toBe(true);

    let server = connect.unwrap();
    let register = await server.register("/Users/tami5/repos/swift/wordle");

    expect(register.isOk(), "Register request is sent without errors").toBe(true);

    let address = register.unwrap();
    expect(typeof address).toBe("string");

  });
});
