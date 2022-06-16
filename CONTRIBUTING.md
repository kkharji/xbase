# Contributing

Please checkout [milestones](https://github.com/tami5/xbase/milestones) for planned future releases and features.

- [Project Organization](#project-organization)
- [Development Setup](#development-setup)

## Project Organization

Here's an overview of the project architecture to help you contribute.

### [proto] crate

Client/Daemon requests and types definitions.

### [daemon] crate

The main product of [xbase] reside, it handle requets and data defined in [proto] crate.

- [build.rs](./daemon/src/build.rs): build request handler definition.
- [run.rs](./daemon/src/run.rs): run request handler definition.
- [drop.rs](./daemon/src/drop.rs): drop request handler definition.
- [register.rs](./daemon/src/register.rs): register request handler definition.
- [nvim.rs](./daemon/src/nvim.rs): helper methods to interact with running nvim instance.

### [sourcekit] crate

The Helper build server implementing [BSP] protocol required because [sourcekit-lsp] can't define compile arguments required to _jump to definition_ and _symbol definition_.

### [lualib] crate

The lua neovim client library is defined. It is mainly used for convenience as it provide access to [proto] types. This is temporary as in the future, I'd would like to switch to grpc and have grpc code generated for lua.

[sourcekit]: ./sourcekit/
[daemon]: ./daemon/
[lualib]: ./lualib/
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


