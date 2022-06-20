<p align="center">
<img src="./media/logo.png" alt="drawing" style="width:400px;"/>
</P>

## 👁 Overview

An XCode replacement-ish *development environment* that aims to be your reliable XCode alternative to develop exciting new [apple] software products 🚀.

[XBase] enables you to build, watch, and run xcode products from within your favorite editor. It supports running products on iOS, watchOS and tvOS simulators, along with real-time logging, and some lsp features such as auto-completion and code navigation. (for a complete list of features, see [🌟 Features](#-features)).

Furthermore, [XBase] has built-in support for a variety of XCode project generators, which allow you to avoid launching XCode or manually editing '*.xcodeproj' anytime you add or remove files. We strongly advise you to use one ... at least till [XBase] supports adding/removing files and folders, along with other requirements. (For further information, see [💆 Generators](#-generators))

- Watch [XBase] repo to remain up to date on fresh improvements and exciting new features.
- Checkout [Milestones](https://github.com/tami5/xbase/milestones) for planned future features and releases.
- Visit [CONTRIBUTING.md](./CONTRIBUTING.md) to have your setup to start contributing and support the project.

Please be aware that [XBase] is still **WIP**, so don't hesitate to report bugs, ask questions or suggest new exciting features.


## Table of Content

- [🌝 Motivation](#-motivation)
- [🌟 Features](#-features)
- [🛠 Requirements](#-requirements)
- [🎮 Usage](#-usage)
- [🦾 Installation](#-installation)
- [⚙️ Defaults](#-defaults)
- [🩺 Debugging](#-debugging)
- [💆 Generators](#-generators)
- [🎥 Preview](#-preview)

## 🌝 Motivation

I chose to dive into iOS/macOS app development after purchasing an M1 MacBook. However, coming from vim/shell environment and being extremely keyboard oriented, I couldn't handle the transition to a closed sourced, opinionated, mouse-driven development environment. I've considered alternatives like [XVim2] and the built-in vim emulator, however still, I'd catch myself frequently hunting for my mouse.

As a long-time vim user who has previously developed a several lua/nvim plugins, I decided to invest some effort in simplifying my development workflow for producing 'xOS' products.

## 🌟 Features

* **Auto-Completion and Code navigation**\
   [sourcekit-lsp] does not support auto-indexing. As a result, [XBase] contains a method for regenerating compilation instructions on directory changes (i.e. file removal/addition), as well as a custom build server that assists [sourcekit-lsp] in providing code navigation and auto complation for project symbols.
* **Multi-nvim instance support**\
    Thanks to having a single daemon running and simple client/server architecture, users can have and use multiple nvim instance running without process duplications and shared state. For instance, stop a watch service that was being run from a different instance.
* **Auto-start/stop main background daemon**\
    There is no need to manually start/stop the daemon background. The daemon will start and end automatically based on the number of connected client instances.
* **Multi Target/Project Support**\
    Users can work on many projects at the same time and build/run on root directory changes or once. TODO
* **Simulator Support**\
    Pick a simulator device that is close to the target's platform and run the target only once or whenever a file changes. Furthermore, if the simulator is not already running, the service will launch the simulator before installing and running your app.
* **Logging buffer**\
    Real-time recording of 'print()' commands and real-time build logs Currently, logs are unique to the requested client; if you want shared logs, please submit an issue.
* **Statusline Support**\
    Global variable to update statusline with build/run commands + [feline] provider. Other statusline plugins support are welcomed.
* **Zero Footprint**\
    Light resource usage. I've been using [XBase] for a while; it typically uses 0.1 percent RAM and 0 percent CPU.
* **XcodeGen Support**\
    Automatically generate new xcodeproj when you edit

## 🛠 Requirements

- [neovim] v0.7.0 or nightly
- [rust] 1.60.0 or up (see [rust getting started])
- [telescope.nvim]
- [plenary.nvim]

## 🎮 Usage

TLDR:
- [Install XBase](#-installation)
- run `require'xbase'.setup({ --[[ see default configuration ]]  })`
- Open xcodeproj codebase.
- Wait for first time project setup finish.
- Start coding
- Use available actions which can be configure with shortcuts bellow

When you start a neovim instance with a root that contains `project.yml,` `Project.swift,` or
`*.xcodeproj,` the daemon server will auto-start if no instance is running, and register the
project once for recompile-watch. To communicate with your deamon, checkout the configurable
shortcuts.


## 🦾 Installation

To install [XBase] on your system you need run `make install`. This will run `cargo build
--release` on all the required binaries in addition to a lua library. The binaries will be
moved to `path/to/repo/bin` and the lua library will be moved to
`path/to/repo/lua/libxbase.so`.

#### With [packer]
```lua
use {
  'tami5/xbase',
    run = 'make install',
    requires = {
      "nvim-lua/plenary.nvim",
      "nvim-telescope/telescope.nvim"
    },
    config = function()
      require'xbase'.setup({})  -- see default configuration bellow
    end
}
```

#### With [vim-plug]
```vim
Plug 'nvim-lua/plenary.nvim'
Plug 'nvim-telescope/telescope.nvim'
Plug 'tami5/xbase', { 'do': 'make install' }
lua require'xbase'.setup()
```

#### With [dein]
```vim
call dein#add('nvim-lua/plenary.nvim')
call dein#add('nvim-telescope/telescope.nvim')
call dein#add('tami5/xbase', { 'build': 'make install' })
lua require'xbase'.setup()
```


## ⚙️ Defaults
```lua
-- NOTE: Defaults
{
  --- Log level. Set to error to ignore everything: { "trace", "debug", "info", "warn", "error" }
  log_level = "debug",
  --- Default log buffer direction: { "horizontal", "vertical", "float" }
  default_log_buffer_direction = "horizontal",
  --- Statusline provider configurations
  statusline = {
    watching = { icon = "", color = "#1abc9c" },
    running = { icon = "⚙", color = "#e0af68" },
    device_running = { icon = "", color = "#4a6edb" },
    success = { icon = "", color = "#1abc9c" },
    failure = { icon = "", color = "#db4b4b" },
    show_progress = false,
  },
  --- TODO(nvim): Limit devices platform to select from
  simctl = {
    iOS = {
      "iPhone 13 Pro",
      "iPad (9th generation)",
    },
  },
  mappings = {
    --- Whether xbase mapping should be disabled.
    enable = true,
    --- Open build picker. showing targets and configuration.
    build_picker = "<leader>b", --- set to 0 to disable
    --- Open run picker. showing targets, devices and configuration
    run_picker = "<leader>r", --- set to 0 to disable
    --- Open watch picker. showing run or build, targets, devices and configuration
    watch_picker = "<leader>s", --- set to 0 to disable
    --- A list of all the previous pickers
    all_picker = "<leader>ef", --- set to 0 to disable
    --- horizontal toggle log buffer
    toggle_split_log_buffer = "<leader>ls",
    --- vertical toggle log buffer
    toggle_vsplit_log_buffer = "<leader>lv",
  }
}
```

## 💆 Generators

Currently, [XBase] supports two project generators: [XcodeGen] and [Tuist]. Support is compes
in form of recompiling the project and regenerating 'xcodeproj' everytime the generator config file is
modified.

## 🩺 Debugging

### Read logs
```bash
# Daemon logs
tail -f /tmp/xbase-daemon.log
# Build Server logs
tail -f /tmp/xbase-server.log
```

## 🎥 Preview

Watch build service.

![](./media/statusline_watch.gif)

On error it opens a log buffer where you can inspect what went wrong, otherwise only the
statusline get updated.

[xcodegen]: https://github.com/yonaskolb/XcodeGen
[sourcekit-lsp]: https://github.com/apple/sourcekit-lsp
[XBase]: https://github.com/tami5/xbase
[xcodebuild]: https://github.com/tami5/xcodebuild
[feline]: https://github.com/feline-nvim/feline.nvim
[XVim2]: https://github.com/XVimProject/XVim2
[rust]: https://www.rust-lang.org
[tuist]: https://github.com/tuist/tuist
[dein]: https://github.com/Shougo/dein.vim
[packer]: https://github.com/wbthomason/packer.nvim
[vim-plug]: https://github.com/junegunn/vim-plug
[rust getting started]: https://www.rust-lang.org/tools/install
[telescope.nvim]: https://github.com/nvim-telescope/telescope.nvim
[plenary.nvim]: https://github.com/nvim-lua/plenary.nvim
[neovim]: https://github.com/neovim/neovim
[tuist]: https://github.com/tuist/tuist

