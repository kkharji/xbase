# Contributing

Please checkout [milestones](https://github.com/tami5/xbase/milestones) for planned future releases and features.

- [Project Organization](#project-organization)
- [Development Setup](#development-setup)

## Project Organization

Here's an overview of the project architecture to help you contribute.

### [proto] crate

Client/Daemon requests and types definitions.

### [daemon] crate

The main product of [xbase] reside, it handle requests and data defined in [proto] crate.

- [build.rs](./daemon/src/build.rs): build request handler definition.
- [run.rs](./daemon/src/run.rs): run request handler definition.
- [drop.rs](./daemon/src/drop.rs): drop request handler definition.
- [register.rs](./daemon/src/register.rs): register request handler definition.
- [nvim.rs](./daemon/src/nvim.rs): helper methods to interact with running nvim instance.
- [project.rs](./daemon/src/project/mod.rs): traits to be implement for each supported project setup.
  - [xcodegen.rs](./daemon/src/project/xcodegen.rs): implantation of project traits for xcodegen projects.
  - [tuist.rs](./daemon/src/project/tuist.rs): implantation of project traits for tuist projects.
  - [barebone.rs](./daemon/src/project/barebone.rs): implantation of project traits for barebone (i.e. no generators) projects.
  - [swift.rs](./daemon/src/project/swift.rs): implantation of project traits for swift (i.e. package.swift) projects.

### [sourcekit] crate

The Helper build server implementing [BSP] protocol required because [sourcekit-lsp] can't define compile arguments required to _jump to definition_ and _symbol definition_.

### [editor] crate

The neovim editor library. provide convenient and backed function to run in neovim runtime

[sourcekit]: ./sourcekit/
[daemon]: ./daemon/
[editor]: ./editor/
[proto]: ./proto/
[xbase]: https://github.com/tami5/xbase
[BSP]: https://build-server-protocol.github.io
[sourcekit-lsp]: https://github.com/apple/sourcekit-lsp

## Development Setup

#### Clone the repo

```sh
git clone https://github.com/tami5/xbase
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
