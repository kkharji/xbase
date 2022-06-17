# Changelog
## [unreleased]
### <!-- 0 -->Features
- `(Build)` <a href="https://github.com/tami5/xbase/commit/64f7591"> Always allow provisioning updates</a>
### <!-- 1 -->Bug Fixes
- `(Lua)` <a href="https://github.com/tami5/xbase/commit/7ff5e0d"> Xclog is not defined</a>
- `(General)` <a href="https://github.com/tami5/xbase/commit/9a13cc8"> Xcodegen binary not found</a>
### <!-- 2 -->Refactor
- `(General)` <a href="https://github.com/tami5/xbase/commit/26ffb32"> Update logging and compile commands (xclog) (#70)</a>
- `(General)` <a href="https://github.com/tami5/xbase/commit/3c53d44"> Separate concerns + contribution guidelines (#76)</a>

## [0.1.2] - 2022-06-11
### <!-- 0 -->Features
- `(General)` <a href="https://github.com/tami5/xbase/commit/b6432a8"> Work on stable rust channel</a>
### <!-- 1 -->Bug Fixes
- `(General)` <a href="https://github.com/tami5/xbase/commit/c9632fa"> Wrong version of xcodebuild + more unstable code removal</a>

## [0.1.1] - 2022-05-30
### <!-- 0 -->Features
- `(Logger)` <a href="https://github.com/tami5/xbase/commit/572f787"> Different indicator for running process</a>
### <!-- 1 -->Bug Fixes
- `(Logger)` <a href="https://github.com/tami5/xbase/commit/25a4e56"> Incorrect title for build once requests</a>
- `(Logger)` <a href="https://github.com/tami5/xbase/commit/4345ae4"> Force scroll</a>
### <!-- 2 -->Refactor
- `(Logger)` <a href="https://github.com/tami5/xbase/commit/f32fd6b"> Append logger title to all msgs</a>
- `(Logger)` <a href="https://github.com/tami5/xbase/commit/8f472b1"> Print exit code</a>
### <!-- 3 -->Enhancement
- `(Logger)` <a href="https://github.com/tami5/xbase/commit/c545b73"> Output readability and traceability</a>

## [0.1.0] - 2022-05-26
### <!-- 0 -->Features
- `(Build)` <a href="https://github.com/tami5/xbase/commit/611ef4d"> Pretty print</a>
- `(Compile)` <a href="https://github.com/tami5/xbase/commit/8855851"> Generate compile flags for files</a>
- `(Compile)` <a href="https://github.com/tami5/xbase/commit/96b44f4"> Use xcodebuild crate</a>
- `(Compile)` <a href="https://github.com/tami5/xbase/commit/8324260"> Panic when compiled commands is empty</a>
- `(Config)` <a href="https://github.com/tami5/xbase/commit/845d62c"> Support custom ignore patterns</a>
- `(Core)` <a href="https://github.com/tami5/xbase/commit/184723e"> Setup watcher</a>
- `(Core)` <a href="https://github.com/tami5/xbase/commit/7e83a55"> Ignore more directories from watcher</a>
- `(Core)` <a href="https://github.com/tami5/xbase/commit/03eece2"> Support watching renames and fix duplicated events</a>
- `(Core)` <a href="https://github.com/tami5/xbase/commit/f161028"> Regenerate .xcodeproj on directory change</a>
- `(Core)` <a href="https://github.com/tami5/xbase/commit/c0df81c"> Ignore files/directories in .gitignore</a>
- `(Core)` <a href="https://github.com/tami5/xbase/commit/1d0f012"> Watcher ignore buildServer.json</a>
- `(Core)` <a href="https://github.com/tami5/xbase/commit/8d7a704"> Generate compile commands on dir change</a>
- `(Core)` <a href="https://github.com/tami5/xbase/commit/b0362ee"> Abstract everything in core</a>
- `(Core)` <a href="https://github.com/tami5/xbase/commit/438cf5a"> Customize root based on target and configuration</a>
- `(Daemon)` <a href="https://github.com/tami5/xbase/commit/8798b62"> Build configuration</a>
- `(Daemon)` <a href="https://github.com/tami5/xbase/commit/45baa47"> List devices and pick device to run</a>
- `(Daemon)` <a href="https://github.com/tami5/xbase/commit/9288671"> Auto-close old processes</a>
- `(Daemon)` <a href="https://github.com/tami5/xbase/commit/38f14e4"> Ensure new projects server support</a>
- `(Error)` <a href="https://github.com/tami5/xbase/commit/0f54252"> Support server errors</a>
- `(Install)` <a href="https://github.com/tami5/xbase/commit/3d32fbb"> Setup installation with make install</a>
- `(Nvim)` <a href="https://github.com/tami5/xbase/commit/d5226d0"> Log generating compile database on nvim</a>
- `(Nvim)` <a href="https://github.com/tami5/xbase/commit/12336ff"> Get project info</a>
- `(Nvim)` <a href="https://github.com/tami5/xbase/commit/56c8326"> Auto update state project</a>
- `(Nvim)` <a href="https://github.com/tami5/xbase/commit/a9eede7"> Build and append log to nvim buffer</a>
- `(Nvim)` <a href="https://github.com/tami5/xbase/commit/a6d7251"> Control what direction to open log buf</a>
- `(Nvim)` <a href="https://github.com/tami5/xbase/commit/9b888d9"> More descriptive serialize error</a>
- `(Nvim)` <a href="https://github.com/tami5/xbase/commit/652ef19"> Basic highlighting of xcodebuild logs</a>
- `(Nvim)` <a href="https://github.com/tami5/xbase/commit/0e41b62"> Log buffer auto-scroll and keep focus</a>
- `(Nvim)` <a href="https://github.com/tami5/xbase/commit/21a9674"> Command_plate</a>
- `(Nvim)` <a href="https://github.com/tami5/xbase/commit/1f08a63"> Multi picker</a>
- `(Nvim)` <a href="https://github.com/tami5/xbase/commit/0dcd297"> Support picker buffer direction</a>
- `(Nvim)` <a href="https://github.com/tami5/xbase/commit/2487351"> Only present device runners for given platform only</a>
- `(Nvim)` <a href="https://github.com/tami5/xbase/commit/bf13348"> Try to minimize effect on startup</a>
- `(Nvim)` <a href="https://github.com/tami5/xbase/commit/85ebe86"> Mappings configuration</a>
- `(Nvim)` <a href="https://github.com/tami5/xbase/commit/8059b24"> Toggle log buffer</a>
- `(Run)` <a href="https://github.com/tami5/xbase/commit/a0531a3"> Run project from neovim</a>
- `(Run)` <a href="https://github.com/tami5/xbase/commit/de4feb1"> Macos app and in buffer logging</a>
- `(Run)` <a href="https://github.com/tami5/xbase/commit/a7ec630"> Run and watch for changes</a>
- `(Runner)` <a href="https://github.com/tami5/xbase/commit/78ea10e"> Log simctl app outputs like print</a>
- `(Runner)` <a href="https://github.com/tami5/xbase/commit/710f450"> Ensure Simulator app is running</a>
- `(Server)` <a href="https://github.com/tami5/xbase/commit/1834662"> Setup</a>
- `(Server)` <a href="https://github.com/tami5/xbase/commit/3f6428e"> Handle errors</a>
- `(Watch)` <a href="https://github.com/tami5/xbase/commit/2d1123e"> Rebuild project on file modification</a>
- `(Watch)` <a href="https://github.com/tami5/xbase/commit/10436b2"> Stop watch build/run service for dropped client</a>
- `(Watch)` <a href="https://github.com/tami5/xbase/commit/b8b74a1"> Update statusline + feline provider</a>
- `(Workspace)` <a href="https://github.com/tami5/xbase/commit/d256c49"> Auto generate .compile when absent</a>
- `(General)` <a href="https://github.com/tami5/xbase/commit/4543d31"> Unregister client on nvim exist + drop workspace</a>
- `(General)` <a href="https://github.com/tami5/xbase/commit/7822b4a"> Retry xcodegen at least three times.</a>
- `(General)` <a href="https://github.com/tami5/xbase/commit/b3bc94b"> Make code a bit more safe (no unwraps)</a>
- `(General)` <a href="https://github.com/tami5/xbase/commit/04ba828"> Multi target watcher</a>
### <!-- 1 -->Bug Fixes
- `(Build)` <a href="https://github.com/tami5/xbase/commit/31abafa"> Remove pseudocode</a>
- `(Build)` <a href="https://github.com/tami5/xbase/commit/7392cfa"> Stream some time doesn't produce Build success</a>
- `(Core)` <a href="https://github.com/tami5/xbase/commit/d9c0296"> On project.yml write, xcodegen errors</a>
- `(Core)` <a href="https://github.com/tami5/xbase/commit/65a5654"> XcodeGen couldn’t be copied to “demo”</a>
- `(Daemon)` <a href="https://github.com/tami5/xbase/commit/ef16dc4"> Project update clear project root</a>
- `(Daemon)` <a href="https://github.com/tami5/xbase/commit/737b7db"> Auto-start</a>
- `(Logger)` <a href="https://github.com/tami5/xbase/commit/e0235a7"> Redundant window open</a>
- `(Nvim)` <a href="https://github.com/tami5/xbase/commit/d8d4ef3"> Daemon commands</a>
- `(Recompile)` <a href="https://github.com/tami5/xbase/commit/4abadd6"> Not skipping file modifications</a>
- `(Run)` <a href="https://github.com/tami5/xbase/commit/a3a8edd"> Running wrong correct output</a>
- `(Run)` <a href="https://github.com/tami5/xbase/commit/a0014e0"> Stop watch service</a>
- `(Runner)` <a href="https://github.com/tami5/xbase/commit/0f26775"> Simctl runner</a>
- `(Server)` <a href="https://github.com/tami5/xbase/commit/697d99f"> Sourcekit panic from swift-frontend compiled command</a>
- `(Xcodebuild)` <a href="https://github.com/tami5/xbase/commit/a366584"> Ignoring build failure</a>
- `(General)` <a href="https://github.com/tami5/xbase/commit/9b53d44"> Recognize project.yml update</a>
### <!-- 2 -->Refactor
- `(Build)` <a href="https://github.com/tami5/xbase/commit/ccada27"> Include watch option instead of special request</a>
- `(Compile)` <a href="https://github.com/tami5/xbase/commit/ec35b61"> Use Pathbuf instead of String</a>
- `(Core)` <a href="https://github.com/tami5/xbase/commit/0c21e7b"> Move tracing install to util</a>
- `(Core)` <a href="https://github.com/tami5/xbase/commit/ad1fffd"> Rename server to daemon</a>
- `(Core)` <a href="https://github.com/tami5/xbase/commit/08d1aed"> Use const for command string name</a>
- `(Core)` <a href="https://github.com/tami5/xbase/commit/8694c47"> Use try_into instead of new</a>
- `(Core)` <a href="https://github.com/tami5/xbase/commit/a201b2e"> Extract code to xcodegen module</a>
- `(Core)` <a href="https://github.com/tami5/xbase/commit/48ad009"> Clean up</a>
- `(Core)` <a href="https://github.com/tami5/xbase/commit/3dd984b"> No objects for stateless functions</a>
- `(Core)` <a href="https://github.com/tami5/xbase/commit/1df0668"> Restructure</a>
- `(Daemon)` <a href="https://github.com/tami5/xbase/commit/96059c4"> Move state to daemon module</a>
- `(Daemon)` <a href="https://github.com/tami5/xbase/commit/be4b5ed"> Organization</a>
- `(Daemon)` <a href="https://github.com/tami5/xbase/commit/e1ac069"> Resuse serde and lua for serialization</a>
- `(Daemon)` <a href="https://github.com/tami5/xbase/commit/3835680"> Point to project nvim script method</a>
- `(Daemon)` <a href="https://github.com/tami5/xbase/commit/5b635da"> Remove project_info request</a>
- `(Daemon)` <a href="https://github.com/tami5/xbase/commit/7307641"> Nvim logger</a>
- `(Daemon)` <a href="https://github.com/tami5/xbase/commit/73cbaf2"> Improve nvim.logger interface</a>
- `(Daemon)` <a href="https://github.com/tami5/xbase/commit/1139fcc"> Always include address in Client</a>
- `(Daemon)` <a href="https://github.com/tami5/xbase/commit/b7a947a"> Move runner related code to new module</a>
- `(Daemon)` <a href="https://github.com/tami5/xbase/commit/15b4c1c"> Switch to process-stream</a>
- `(Daemon)` <a href="https://github.com/tami5/xbase/commit/6ee4138"> Abstract some logic in Client struct</a>
- `(Drop)` <a href="https://github.com/tami5/xbase/commit/4debb3b"> Remove debug code</a>
- `(Logger)` <a href="https://github.com/tami5/xbase/commit/fa52bf5"> Log and trace at the same time</a>
- `(Logger)` <a href="https://github.com/tami5/xbase/commit/0cc1e09"> Don't require win to be passed around</a>
- `(Logger)` <a href="https://github.com/tami5/xbase/commit/d6e9ffb"> Initialization arguments</a>
- `(Nvim)` <a href="https://github.com/tami5/xbase/commit/db76c64"> Echo instead of log for success</a>
- `(Project)` <a href="https://github.com/tami5/xbase/commit/61b955d"> Make config optional</a>
- `(Runner)` <a href="https://github.com/tami5/xbase/commit/9326c3b"> Merge into one file</a>
- `(Runner)` <a href="https://github.com/tami5/xbase/commit/295d8b1"> Rewrite and update simctl runner and log format</a>
- `(Runner)` <a href="https://github.com/tami5/xbase/commit/0313b76"> Simplify api</a>
- `(Runner)` <a href="https://github.com/tami5/xbase/commit/7245899"> Better architecture</a>
- `(Server)` <a href="https://github.com/tami5/xbase/commit/4aacd34"> Docs + readability</a>
- `(Types)` <a href="https://github.com/tami5/xbase/commit/e7915e8"> Move build types to it's own file</a>
- `(Util)` <a href="https://github.com/tami5/xbase/commit/5a4a79b"> Create pid and fmt module</a>
- `(Watch)` <a href="https://github.com/tami5/xbase/commit/79cfc15"> Custom error type to control watch service state</a>
- `(Watch)` <a href="https://github.com/tami5/xbase/commit/c3df803"> Rework watching logic</a>
- `(Watcher)` <a href="https://github.com/tami5/xbase/commit/b3972f7"> Improve code readability</a>
- `(General)` <a href="https://github.com/tami5/xbase/commit/970932c"> Move shared to core as lib</a>
- `(General)` <a href="https://github.com/tami5/xbase/commit/17612be"> Re-structure codebase</a>
- `(General)` <a href="https://github.com/tami5/xbase/commit/90755c8"> Nest directories based on use context</a>
- `(General)` <a href="https://github.com/tami5/xbase/commit/a5c42fa"> Constrain modules with feature flags + custom bin/lib paths</a>
- `(General)` <a href="https://github.com/tami5/xbase/commit/a00ac2e"> Compilation module</a>
- `(General)` <a href="https://github.com/tami5/xbase/commit/8dd319e"> Make tracing to stdout optional</a>
- `(General)` <a href="https://github.com/tami5/xbase/commit/7da58b6"> Rename daemon command to requests</a>
- `(General)` <a href="https://github.com/tami5/xbase/commit/8430976"> Support multiple target watch</a>
- `(General)` <a href="https://github.com/tami5/xbase/commit/33e2528"> Rename project to xbase</a>
### <!-- 3 -->Enhancement
- `(Error)` <a href="https://github.com/tami5/xbase/commit/b3f90e2"> Custom result and error type</a>
- `(Logger)` <a href="https://github.com/tami5/xbase/commit/78f587d"> Simplify</a>
- `(Nvim)` <a href="https://github.com/tami5/xbase/commit/8ca628e"> Add highlight to error</a>
- `(Watcher)` <a href="https://github.com/tami5/xbase/commit/af1a390"> Features guards</a>
- `(Watcher)` <a href="https://github.com/tami5/xbase/commit/18697f0"> Add event debounce</a>
- `(Xcodegen)` <a href="https://github.com/tami5/xbase/commit/2238bb6"> Types</a>
### Documentation
- `(General)` <a href="https://github.com/tami5/xbase/commit/7045049"> Update compile module</a>
- `(General)` <a href="https://github.com/tami5/xbase/commit/315f62d"> Add missing doc strings</a>
### Core
- `(General)` <a href="https://github.com/tami5/xbase/commit/87d36fd"> Initial setup</a>
### Deps
- `(General)` <a href="https://github.com/tami5/xbase/commit/8de6112"> Point to remote packages</a>
### Dev
- `(Lua)` <a href="https://github.com/tami5/xbase/commit/cc06934"> Add lua types</a>
- `(General)` <a href="https://github.com/tami5/xbase/commit/a87a4ff"> Auto convert todo comments @alstr</a>
- `(General)` <a href="https://github.com/tami5/xbase/commit/0b86ef7"> Watch commands</a>
### Revert
- `(General)` <a href="https://github.com/tami5/xbase/commit/5d382fe"> Using gitignore.rs</a>

