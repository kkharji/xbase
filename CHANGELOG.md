# Changelog
## 🔥 [Unreleased](https://github.com/tami5/xbase)
### <!-- 0 -->Features
<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/a307197fc66836e2a3de44bf74879627d03b8760">Use vim.notify</a></b> <code>#editor</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/73801509728418db3a6a3633ed6b189766ad11c3">Support xcworkspace (#101)</a></b> <code>#general</code> <u><b>....</b></u></summary><br />

- When xcworkspace exists, use it instead of xcodeproj when compiling and recompiling projects.
- When xcworkspace exists, build target are passed with `-scheme` flag, so targets and scheme need to have the same name.
- speed up tuist setup through compiling the Manifest scheme instead of each target</details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/d9a2c288bd3838965b82c970334e143a0543a68a">Support multiple projects within a single instance</a></b> <code>#general</code></summary></details></dd></dl>

### <!-- 1 -->Bug Fixes
<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/bace0e278b3d009da7cd6ade4cc4b21155e1c802">Remove old logging interface</a></b> <code>#nvim</code> <u><b>....</b></u></summary><br />

This errors when the users add no longer supported or invalid configuration key</details></dd></dl>

### <!-- 2 -->Refactor
<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/526e17ceaa5bdaf70df634a04eff08849a09d29c">Move out nvim specific logic (#103)</a></b> <code>#daemon</code> <u><b>....</b></u></summary><br />

* init

* chore(deps): update xclog and process-stream + refactor

* ref: setup shared logger

* ref: remove nvim-rs

* feat: broadcast server

* fix(editor): receiving multiple messages at the same time

This just a hack because I couldn't pinpoint why is the client is
receiving a bulk of message separated by newline

* ref(editor): rename BroadcastMessage to Broadcast

* feat(nvim): setup logger

* fix: run/build commands

* ref: logs

* ref: remove log macros

* ref: remove log_request

* ref: remove client type, use root only

* fix: status line updates

* ref: rename editor to client

* fix: watch status

* feat(nvim): support custom notify

* feat: respect user log level

* enh(logger): format

* fix(tuist): generate compile commands

* ref: rename neovim to nvim

* chore: cleanup

* ref: move make try_register part of register

* ref(client): register return bool

* ref: move logging functionality to lua

* ref: clean up

* fix: open logger on error

* feat: append generation logs on error only

* ref(nvim): move logger buffer mappings to setup

* fix(nvim): change log buffer change position if already opened

* feat(nvim): add custom configurations for log_buffer

* chore: add icon to error messages

* feat(messages): success level

* feat: update lsp server on compile files reloaded</details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/d5b4ea1ec33d6aea4f13f869cfcdcdb6a105b86e">Switch to tarpc framework</a></b> <code>#general</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/f45bfbd40eb20cc7ced7c79943331bb0af8e2368">Rename lualib to editor-lib</a></b> <code>#general</code></summary></details></dd></dl>

### <!-- 3 -->Enhancement
<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/99340ce2c5185c5fa85c2c8d181e467be278f240">Formatting, display and readability</a></b> <code>#logger</code></summary></details></dd></dl>


## 🎉 [v0.2.0](https://github.com/tami5/xbase/tree/v0.2.0) - 2022-06-22
### <!-- 0 -->Features
<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/64f75911500ccce0d2771d0095a3e03189a2d99c">Always allow provisioning updates</a></b> <code>#build</code> <u><b>....</b></u></summary><br />

Hot fix for an issues I had where I needed to open xcode for updating my
signature</details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/8837bca257b2e86bc019a1c6c4e33b543080ebf8">Faster build</a></b> <code>#cargo</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/472bf59380b992e21589d08bcdb56037804d9544">Generate compile commands without signing</a></b> <code>#compile</code> <u><b>....</b></u></summary><br />

Finally, this will ensure no more errors with regards to provisioning
profile or singing, at least for auto complaining</details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/0a2943b628cd0478156962569e6fdc812bd44421">Respect gitignore</a></b> <code>#daemon</code> <u><b>....</b></u></summary><br />

previously it was required to set custom paths to ignore in project.yml,
now extra ignored path reads from gitignore instead.</details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/ba9f2976fdeccc882650b3d71a5527de753c72f5">Update status variable when watch is running</a></b> <code>#nvim</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/ef92912acc2f4ea0c8dbc719b2b18cba6f12a24a">Reload sourcekit server on compile</a></b> <code>#nvim</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/7fdcfef313c9500cac38ebec1135b5e0bc329f24">Clear state on .compile change</a></b> <code>#sourcekit</code> <u><b>....</b></u></summary><br />

Doesn't seem critical now that the sourcekit lsp server is reloaded on compile.</details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/1f1b3ad80ec26557b0a2fd94e50de7ceecd63239">Init dependabot</a></b> <code>#general</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/7fb498cadc813df1941665398b5325a255d6015e">Make xcodeproj source of truth (#80)</a></b> <code>#general</code> <u><b>....</b></u></summary><br />

* feat(daemon): switch to reading from xcodeproj only
* ref(daemon): remove xcodegen project types
* ref: remove xcodegen.rs
* feat(lua): identity xcodeproj on setup
* fix(compile): error while removing non-existing cache
* chore(readme): requirements
* ref: use platform target method instead of sdkroots
* ref: use xcodeproj new api
* chore(deps): bump wax dependency
* chore: update readme</details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/79ea745d5c673f1ff717bf3f097bc661713b562a">Support tuist (#91)</a></b> <code>#general</code> <u><b>....</b></u></summary><br />

* feat(tuist): support regeneration
* feat(project): support generating xcodeproj when absent
* feat(compile): append xcodeproj generation logs
* ref(compile): check for xcodeproj before trying to generate it
* feat(tuist): generate both project and manifest xcodeproj
* feat(tuist): generate compile commands for both project and manifest
* feat(nvim): update status variable when watch is running
* ref(project): decompose and specialize
* feat(tuist): lsp support for tuist files
* chore(readme): update
* ref: make main binary named xbase
* feat(tuist): recompile on config files change
* fix(xcodegen): ignoring existing xcodeproj
* fix(compile): on file rename</details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/a3703a05d876e8516a285186d5f8aaf55f07a26b">Support swift projects (#97)</a></b> <code>#general</code> <u><b>....</b></u></summary><br />

* feat(swift): initial support 
closes #66 
* ref(daemon): abstract run logic
* feat(swift): run project
* fix(swift): logger
* chore(readme): update
* chore(ci): update ci command
* feat(swift): ignore tests target for build and run</details></dd></dl>

### <!-- 1 -->Bug Fixes
<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/7466c267be2d848ba0314ebe2d3e700d5086772b">Error while removing non-existing cache</a></b> <code>#compile</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/c7d241397ea8abb5b580fbc0df9bc124690d0221">Incorrect paths to binaries</a></b> <code>#daemon</code> <u><b>....</b></u></summary><br />

CARGO_MANIFEST_DIR unfortunately points to package root instead of
workspace root</details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/bdd0e6ce03fcd76c29cd418755d8f9c68174cf63">Crashing on multiline message nvim print</a></b> <code>#daemon</code> <u><b>....</b></u></summary><br />

only print the first line and the rest redirect to log buffer</details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/207ea482b8c1941e9e920e68241d00444d3351c1">Avoid adding extra `/`</a></b> <code>#gitignore</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/984f94022805f1c12b1fb11af9596ee1f3320576">Avoid duplicating **</a></b> <code>#gitignore</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/a9b034201ee69c5a6d649ad0d7c682d060e9f550">Fix simulator latency</a></b> <code>#logging</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/7ff5e0da566f35cf60a3377c674c6e44491a6276">Xclog is not defined</a></b> <code>#lua</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/9a13cc88f3f79ef3c2701d42108c53e6cc19133e">Xcodegen binary not found</a></b> <code>#general</code></summary></details></dd></dl>

### <!-- 2 -->Refactor
<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/8e5e4e5f62ede9c5596708ad4ed2f906dc678981">Abstract build logic into ProjectBuild</a></b> <code>#daemon</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/26ffb3291f3e6c005aede3c66a05d9c50fc7bc09">Update logging and compile commands (xclog) (#70)</a></b> <code>#general</code> <u><b>....</b></u></summary><br />

- switch to new [xclog](https://github.com/tami5/xclog) api and refractor duplicated code.
- remove xcode.rs module
- Fix #69.</details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/3c53d443373f63cf9331853b506f86d168d00395">Separate concerns + contribution guidelines (#76)</a></b> <code>#general</code> <u><b>....</b></u></summary><br />

* ref: extract tracing setup to lib/tracing
* ref: extract build server to sourcekit crate
* ref: extract lib and daemon
* ref(daemon): use xbase_proto
* ref: flatten structure
* feat: contributing guidelines
* chore: update readme</details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/52739ad70820ee2d8056dae3b462cc51f1d48868">Remove crossbeam-channel crate</a></b> <code>#general</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/f0bced1fdce68079151675b6f9e5d947e3159356">Move project/device module to root</a></b> <code>#general</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/c28a0ef78e6302eabdb4cbdba44fe68eb521b7c5">Rename tracing package to log</a></b> <code>#general</code> <u><b>....</b></u></summary><br />

conflicts with running cargo check and test</details></dd></dl>

### Ci
<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/05a05a42e8f76181d301f2408c84ce138ddb4d6c">Build, install and format</a></b> <code>#general</code></summary></details></dd></dl>


## 🎉 [v0.1.2](https://github.com/tami5/xbase/tree/v0.1.2) - 2022-06-11
### <!-- 0 -->Features
<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/b6432a80fb6586b04e5dda1c0bef09a28983573d">Work on stable rust channel</a></b> <code>#general</code> <u><b>....</b></u></summary><br />

closes https://github.com/tami5/xbase/issues/68</details></dd></dl>

### <!-- 1 -->Bug Fixes
<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/c9632fa70cc068202edb8829170d8506e0d04cbf">Wrong version of xcodebuild + more unstable code removal</a></b> <code>#general</code></summary></details></dd></dl>


## 🎉 [v0.1.1](https://github.com/tami5/xbase/tree/v0.1.1) - 2022-05-30
### <!-- 0 -->Features
<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/572f787f67b92ab7296cf635a54358ee5b50c9c2">Different indicator for running process</a></b> <code>#logger</code></summary></details></dd></dl>

### <!-- 1 -->Bug Fixes
<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/25a4e561ddf892f5f332b4ba13d2825f6fa18625">Incorrect title for build once requests</a></b> <code>#logger</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/4345ae44a5842d9a848813e114ca3ee2c10c25f1">Force scroll</a></b> <code>#logger</code></summary></details></dd></dl>

### <!-- 2 -->Refactor
<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/f32fd6bb6384050f85061308a75a7a1aa19f3ad8">Append logger title to all msgs</a></b> <code>#logger</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/8f472b18ccb52d70149004302c6da75ae991dec9">Print exit code</a></b> <code>#logger</code></summary></details></dd></dl>

### <!-- 3 -->Enhancement
<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/c545b7365a6808d3f744aaf6b82cacf3d62d995d">Output readability and traceability</a></b> <code>#logger</code></summary></details></dd></dl>


## 🎉 [v0.1.0](https://github.com/tami5/xbase/tree/v0.1.0) - 2022-05-26
### <!-- 0 -->Features
<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/611ef4d76f6ebd474283b63a44d134c3b0341d9b">Pretty print</a></b> <code>#build</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/88558513911501de9a3b2aa32aaecf2fed926621">Generate compile flags for files</a></b> <code>#compile</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/96b44f40ddf4946bc3e21185d1983f41c33534c3">Use xcodebuild crate</a></b> <code>#compile</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/83242603254a18f68149fda001a9960e2f81bdcd">Panic when compiled commands is empty</a></b> <code>#compile</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/845d62cc0fcb74d45fc6817ba46b6999773a2ac0">Support custom ignore patterns</a></b> <code>#config</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/184723e5ef35543688eaa595715f760f6bf3c1ff">Setup watcher</a></b> <code>#core</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/7e83a552d6bbbd41dc168e72345c08a67502f22e">Ignore more directories from watcher</a></b> <code>#core</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/03eece2c73f3cc7cd86c80b068ec2611ce0f73f6">Support watching renames and fix duplicated events</a></b> <code>#core</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/f16102822ac50b5b0409acd5ad156128297281eb">Regenerate .xcodeproj on directory change</a></b> <code>#core</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/c0df81cb5235644a56e775fb324667c7a9890508">Ignore files/directories in .gitignore</a></b> <code>#core</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/1d0f0126ac0559c9269fcd95bcf854a52c1e9807">Watcher ignore buildServer.json</a></b> <code>#core</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/8d7a70469f90435139344b544576cfa6e93d036b">Generate compile commands on dir change</a></b> <code>#core</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/b0362eea298b2e149f151536aa65c8971f88c368">Abstract everything in core</a></b> <code>#core</code> <u><b>....</b></u></summary><br />

Feels right todo it this way. more feature guard needs to be added
though.</details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/438cf5ad158c8f419a04134813ea716a73c9b64b">Customize root based on target and configuration</a></b> <code>#core</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/8798b622d4a4841f9e0bb2016d97d7fb207abd84">Build configuration</a></b> <code>#daemon</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/45baa4789da3066d8d8d5d14e25394b0346c01a3">List devices and pick device to run</a></b> <code>#daemon</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/9288671737d9f3b073e19bbe3fbfac116c99e5e0">Auto-close old processes</a></b> <code>#daemon</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/38f14e4c3de8f6e4e847ebaddcb3300a20295540">Ensure new projects server support</a></b> <code>#daemon</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/0f542527d1aa2a7f248fa2be58962d4c235f9172">Support server errors</a></b> <code>#error</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/3d32fbbf4295f34197504a59d526edfc30d78caa">Setup installation with make install</a></b> <code>#install</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/d5226d09d5f90de0186939ba058c1f236451bd43">Log generating compile database on nvim</a></b> <code>#nvim</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/12336ffe729b243a322dbd98a71dfc52129d74a7">Get project info</a></b> <code>#nvim</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/56c8326a62b2910a181b62edbaa413acf1c5e538">Auto update state project</a></b> <code>#nvim</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/a9eede75989425943de53943e00fc1faa23f6d97">Build and append log to nvim buffer</a></b> <code>#nvim</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/a6d7251e6b55efe640853998fe2e1e86f561e9fe">Control what direction to open log buf</a></b> <code>#nvim</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/9b888d9e56f940ec39a42f42ba413572be8da9cb">More descriptive serialize error</a></b> <code>#nvim</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/652ef19c503a0160b946edac12802aa76ee8ce46">Basic highlighting of xcodebuild logs</a></b> <code>#nvim</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/0e41b62a01b0853c7ed498fe3eae4b0ae8f82aba">Log buffer auto-scroll and keep focus</a></b> <code>#nvim</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/21a96744c5f0a2d3cbaefc0d6c13edcad2d1c162">Command_plate</a></b> <code>#nvim</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/1f08a6398ccd818ecfb67ccf9f2ca424378acf00">Multi picker</a></b> <code>#nvim</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/0dcd29779e460d131e78953ac810d6cb077663cf">Support picker buffer direction</a></b> <code>#nvim</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/24873516cc57a18f132ad527e53ca106372e1d0e">Only present device runners for given platform only</a></b> <code>#nvim</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/bf13348bf89fdd4393712a44ce600ee93f7337b1">Try to minimize effect on startup</a></b> <code>#nvim</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/85ebe866f3675871e649a80dee72b628d46b9cd3">Mappings configuration</a></b> <code>#nvim</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/8059b249d9ae3dc1e5060de97b761c9dfa5a7041">Toggle log buffer</a></b> <code>#nvim</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/a0531a3f0ebc4739cff8aa83b381bbcd06664291">Run project from neovim</a></b> <code>#run</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/de4feb1d7e60e1857aef69950e6f5684b1922a54">Macos app and in buffer logging</a></b> <code>#run</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/a7ec630366860a308e5b1e3e4555b004510dfc34">Run and watch for changes</a></b> <code>#run</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/78ea10e6a254a4aba6985f76a98d44c081acd053">Log simctl app outputs like print</a></b> <code>#runner</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/710f45029a5bcd13fcdd3f49a845db596222aa9b">Ensure Simulator app is running</a></b> <code>#runner</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/1834662eb30f844af1c9aaf2ece17db90f446897">Setup</a></b> <code>#server</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/3f6428eeb6816ce1bbe634c71606386c6127d000">Handle errors</a></b> <code>#server</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/2d1123e26527e95776e18b4fc28a50013d63ba7c">Rebuild project on file modification</a></b> <code>#watch</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/10436b20e55c11b957f4fbbe001c49609f55ecc2">Stop watch build/run service for dropped client</a></b> <code>#watch</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/b8b74a1c9dcf3be675140742b5393f19bfe13262">Update statusline + feline provider</a></b> <code>#watch</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/d256c49b0c50b28ef95701a46925443bb35f36db">Auto generate .compile when absent</a></b> <code>#workspace</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/4543d310f1ac0478aff012af7a2c0d86cb39fdb9">Unregister client on nvim exist + drop workspace</a></b> <code>#general</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/7822b4a4063dcce24cdb31d07222ecd20b014d1b">Retry xcodegen at least three times.</a></b> <code>#general</code> <u><b>....</b></u></summary><br />

Getting errors from objc NS side with Xcodegen being already created or it
can't be overwritten.</details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/b3bc94bf90f7d69967f859149dedb60e0b6f37a7">Make code a bit more safe (no unwraps)</a></b> <code>#general</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/04ba82865312e6cea89dc19a030769b5f1f99a64">Multi target watcher</a></b> <code>#general</code></summary></details></dd></dl>

### <!-- 1 -->Bug Fixes
<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/31abafad0f46f32eae63b96915e9251ab93ee461">Remove pseudocode</a></b> <code>#build</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/7392cfaeb5116f8d14e817611367c15fff7cbe27">Stream some time doesn't produce Build success</a></b> <code>#build</code> <u><b>....</b></u></summary><br />

Depending instead on exit code</details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/d9c0296b920ac4bec43862904f95953cc0cf833d">On project.yml write, xcodegen errors</a></b> <code>#core</code> <u><b>....</b></u></summary><br />

"“XcodeGen” couldn’t be copied to “wordle” because an item with the same name already exists."
UserInfo={NSSourceFilePathErrorKey=/var/folders/lm/jgnf6c7941qbrz4r6j5qscx00000gn/T/20E766F9-95D6-42FE-BF73-F5C20430739A-36915-00000DFC9FAEE17B/XcodeGen,
NSUserStringVariant=(Copy),
NSDestinationFilePath=/Users/tami5/repos/test.xcodeproj,
NSFilePath=/var/folders/lm/jgnf6c7941qbrz4r6j5qscx00000gn/T/20E766F9-95D6-42FE-BF73-F5C20430739A-36915-00000DFC9FAEE17B/XcodeGen,
NSUnderlyingError=0x600003a9aac0 {Error Domain=NSPOSIXErrorDomain Code=17 "File exists"}}</details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/65a5654e3f314c4b68235d9936af18c76aea46b5">XcodeGen couldn’t be copied to “demo”</a></b> <code>#core</code> <u><b>....</b></u></summary><br />

This issue started happening out of no where so delaying processing
temporarily until I find a proper solution</details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/ef16dc40ccd789079c10e75616614a4e855478b0">Project update clear project root</a></b> <code>#daemon</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/737b7db2bf660f62d0ca7f719fa7c67473276c2c">Auto-start</a></b> <code>#daemon</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/e0235a7f405e5730a59db59a53aff4c7e6aad2ed">Redundant window open</a></b> <code>#logger</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/d8d4ef3b0c10ae4813e02f9d7577490bebd89f48">Daemon commands</a></b> <code>#nvim</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/4abadd6811c30507bd5bc41ae826b13cda447999">Not skipping file modifications</a></b> <code>#recompile</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/a3a8eddaeafff39e3ef55cfbb0b634177570011f">Running wrong correct output</a></b> <code>#run</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/a0014e0907ad6a5fb1268858d098a21350e05409">Stop watch service</a></b> <code>#run</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/0f267750a42080fb1b3896414fe0c34562d61ef4">Simctl runner</a></b> <code>#runner</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/697d99f3cc9244f4f1680ffdd870565514b6c92e">Sourcekit panic from swift-frontend compiled command</a></b> <code>#server</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/a366584728e8cfee589bd8fbe8770040af784816">Ignoring build failure</a></b> <code>#xcodebuild</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/9b53d44c31147c7f915d0be137beed0edbb8a789">Recognize project.yml update</a></b> <code>#general</code></summary></details></dd></dl>

### <!-- 2 -->Refactor
<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/ccada27ee60db1aacc70c6cb6d9b68122f4ca4ff">Include watch option instead of special request</a></b> <code>#build</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/ec35b61c7e0024b170aef703313ddd9384dd40d2">Use Pathbuf instead of String</a></b> <code>#compile</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/0c21e7bc2334d66b2e985b6b6ac31b45608f23af">Move tracing install to util</a></b> <code>#core</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/ad1fffd0d3a87c93218d804d23028b9326de05ab">Rename server to daemon</a></b> <code>#core</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/08d1aed862bdefe640f9c5adc184a8f33de7ea6c">Use const for command string name</a></b> <code>#core</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/8694c471ad68c99ff400c768f65ec17fa8fe9ead">Use try_into instead of new</a></b> <code>#core</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/a201b2edfae884aaad9eac931241025fa998cec3">Extract code to xcodegen module</a></b> <code>#core</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/48ad0090ba393eb029bc1bab7b6fca349f843998">Clean up</a></b> <code>#core</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/3dd984b0d0ba688fa7c2634f38cd30abea14b746">No objects for stateless functions</a></b> <code>#core</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/1df0668850457959f614de5812426d9f5cf4c135">Restructure</a></b> <code>#core</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/96059c4ae589a807b9edf2cd709bc4e5d4249f04">Move state to daemon module</a></b> <code>#daemon</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/be4b5ed2f86d7a7fd24c0a20f1b08a6c049751a0">Organization</a></b> <code>#daemon</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/e1ac0696e144ea0a1ae34c9402d06e5df509339a">Resuse serde and lua for serialization</a></b> <code>#daemon</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/3835680424e59e821b77802441d4521b9421a15f">Point to project nvim script method</a></b> <code>#daemon</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/5b635da04cf2a6eb668aa8bbc6c2cafc6428c949">Remove project_info request</a></b> <code>#daemon</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/7307641a47d93c6e81d71f65ec95f6bc64c93a12">Nvim logger</a></b> <code>#daemon</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/73cbaf248b828a384263971c4e22c54dfc06766f">Improve nvim.logger interface</a></b> <code>#daemon</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/1139fcc6fd6d4b8ca88ce20955ee385cd2398fe6">Always include address in Client</a></b> <code>#daemon</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/b7a947a97abca5c4ac33abe1d9407e0d23a40c1f">Move runner related code to new module</a></b> <code>#daemon</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/15b4c1c3e31b90cc76c7c6ed93d857a41d349d1f">Switch to process-stream</a></b> <code>#daemon</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/6ee4138bf33c0d4e3122d36ef45b4ead48996fb4">Abstract some logic in Client struct</a></b> <code>#daemon</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/4debb3baddd3e4ec36c543d54604f0b7aba28b50">Remove debug code</a></b> <code>#drop</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/fa52bf55b47f8496a9023c8a35c269a271b2c620">Log and trace at the same time</a></b> <code>#logger</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/0cc1e09d24074252df24177d6ef756110ddd3269">Don't require win to be passed around</a></b> <code>#logger</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/d6e9ffbdfd7bb8e98616a7ae0d426e0e09516b5e">Initialization arguments</a></b> <code>#logger</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/db76c6486774d6ca0aa639fbde03399f2f78bf65">Echo instead of log for success</a></b> <code>#nvim</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/61b955de6cf9b66a042bc0802c5c198219da2a93">Make config optional</a></b> <code>#project</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/9326c3b5469c8cc1075d7d6d9751d584ad97187e">Merge into one file</a></b> <code>#runner</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/295d8b1d6ecf682a619693089b4a501ff718df39">Rewrite and update simctl runner and log format</a></b> <code>#runner</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/0313b767abd7763f2955b95b71e0358159fa94d8">Simplify api</a></b> <code>#runner</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/724589988bb422e4bdcc6aa68869b2053fe5eea6">Better architecture</a></b> <code>#runner</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/4aacd341e7b16accbb5d1cfed52368a5f7065bbf">Docs + readability</a></b> <code>#server</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/e7915e82062fdc27d96a09f97b2abf5f96671c95">Move build types to it's own file</a></b> <code>#types</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/5a4a79b00f8670ddf5fff045bf924edd62ea4f7e">Create pid and fmt module</a></b> <code>#util</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/79cfc15599e41a2df136e99c4fd56eb868f23344">Custom error type to control watch service state</a></b> <code>#watch</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/c3df803aa0a57bdb204675e93c3e9d0bbecce61f">Rework watching logic</a></b> <code>#watch</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/b3972f79a7c18c284e27024b364a9c1f9e0ab4c1">Improve code readability</a></b> <code>#watcher</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/970932c133629462d823f9482ed1a3f69c65c0cb">Move shared to core as lib</a></b> <code>#general</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/17612bea02fc0b33954a141f285b3949c0c341ac">Re-structure codebase</a></b> <code>#general</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/90755c80e9546b051ab182f2e9c118cb19b964a9">Nest directories based on use context</a></b> <code>#general</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/a5c42fabcf8ba548b719e2ce1585ec32ddcc2444">Constrain modules with feature flags + custom bin/lib paths</a></b> <code>#general</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/a00ac2ee6ff1c6e0b7950013edc3cf4c1b32d677">Compilation module</a></b> <code>#general</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/8dd319eca1d4030a7240fc449890ee1fc126fdf1">Make tracing to stdout optional</a></b> <code>#general</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/7da58b6d2b553a6f5c04ff0aad8d87c4bf0d7e96">Rename daemon command to requests</a></b> <code>#general</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/8430976e4bb11be1fc0dda0e486e5ac4ceb47763">Support multiple target watch</a></b> <code>#general</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/33e2528bfe7543b0a4339fa2deb4cbec4898b5b2">Rename project to xbase</a></b> <code>#general</code></summary></details></dd></dl>

### <!-- 3 -->Enhancement
<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/b3f90e2ec4a8d097c89a0a1f44a40a52749497ed">Custom result and error type</a></b> <code>#error</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/78f587d7e8cae77edff5e9f1c209b0d8837738fe">Simplify</a></b> <code>#logger</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/8ca628e6b8058ab37ec7785aba547f6ff9a927b1">Add highlight to error</a></b> <code>#nvim</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/af1a3909a31a5517e5ab4f4452cccd0d5767ab1a">Features guards</a></b> <code>#watcher</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/18697f056844be210b70a65e4088ff0dbd8aaf50">Add event debounce</a></b> <code>#watcher</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/2238bb604d2e6aef2772366e5251be99128aa5fe">Types</a></b> <code>#xcodegen</code></summary></details></dd></dl>

### Documentation
<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/7045049fbd5d2580e7a69ed9e5454f3c5b103d2f">Update compile module</a></b> <code>#general</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/315f62d08d031f24f0a4353e7242e90472ab5c61">Add missing doc strings</a></b> <code>#general</code></summary></details></dd></dl>

### Core
<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/87d36fdf5fd3f7e14469347e1b179d4d9c6ee88e">Initial setup</a></b> <code>#general</code></summary></details></dd></dl>

### Deps
<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/8de6112aa55637d623f3f2facc46030e4bbe975c">Point to remote packages</a></b> <code>#general</code></summary></details></dd></dl>

### Dev
<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/cc06934028f3ede2c0eeca18a1af78202aadc1e6">Add lua types</a></b> <code>#lua</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/a87a4ff04529327cc5d8f8093e91118478b6c44b">Auto convert todo comments @alstr</a></b> <code>#general</code></summary></details></dd></dl>

<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/0b86ef7ea0433ae438b7de0a32caf3908ffc01d3">Watch commands</a></b> <code>#general</code></summary></details></dd></dl>

### Revert
<dl><dd><details><summary><b><a href="https://github.com/tami5/xbase/commit/5d382fe63169c966922c2805bf9d63b1e0dcfd1f">Using gitignore.rs</a></b> <code>#general</code> <u><b>....</b></u></summary><br />

Breaking .. will revisit the issue later</details></dd></dl>


