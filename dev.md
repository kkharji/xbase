# XBase Development

This a quick guide to understand and edit xbase source code.

- [Project Organization](#project-organization)
- [Development Setup](#development-setup)

## Project Organization

Here's an overview of the project architecture to help you contribute.

### [xbase] source code

- [Server Startup and OS Signal Handler Logic `main.rs`](./src/main.rs)
- [State Initialization and Access Logic `state.rs`](./src/state.rs)
- [Server/Client Logic (server/*)](./src/server/mod.rs)
  - [Request Definition and Handler `request.rs`](./src/server/request.rs)
  - [Response Definition `response.rs`](./src/server/response.rs)
  - [Client Handler `stream.rs`](./src/server/stream.rs)
  - [Register Handler `register.rs`](./src/server/register.rs)
  - [Drop Handler `drop.rs`](./src/server/drop.rs)
  - [Run Handler `run.rs`](./src/server/run.rs)
  - [Build Handler `build.rs`](./src/server/build.rs)
  - [Runners Handler `runners.rs`](./src/server/runners.rs)
  - [ProjectInfo Handler `project_info.rs`](./src/server/project_info.rs)
- [Project Running Logic `runner/*`](./src/runner/)
- [Project Watching Logic `watcher/*`](./src/wathcer/)
- [Project Handling Logic `project/*`](./src/project/mod.rs)
  - [XCodeGen Project Support `xcodegen.rs`](./src/project/xcodegen.rs)
  - [Tuist Project Support `tuist.rs`](./src/project/tuist.rs)
  - [Barebone Project Support `barebone.rs`](./src/project/barebone.rs)
  - [Swift Package Support `swift.rs`](./src/project/swift.rs)
- [General Purpose Types `types.rs`](./src/types.rs)
- [Serializable/Deserializable Errors `error.rs`](./src/error.rs)

### [sourcekit-helper] crate

Build server implementing [BSP] protocol required because [sourcekit-lsp] can't define compile arguments required to _jump to definition_ and _symbol definition_.


[sourcekit-helper]: ./crates/sourcekit-helper/
[xbase]: ./src/
[lua]: ./lua/
[xbase]: https://github.com/xbase-lab/xbase
[BSP]: https://build-server-protocol.github.io
[sourcekit-lsp]: https://github.com/apple/sourcekit-lsp

## Development Setup

#### Clone the repo

```sh
git clone https://github.com/xbase-lab/xbase
```

#### Install in debug mode

Do all the setup required to watch and develop

```sh
make install_debug
```

#### Start watchers

Watch all the products and trigger recompile when the source code changes.

```sh
make watch
```
