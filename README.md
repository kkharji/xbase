<p align="center">
<img src="./media/logo.png" alt="drawing" style="width:400px;"/>
</P>

## 👁 Overview

A **WORK IN PROGRESS** Xcode replacement-ish development environment for neovim. [Xbase] currently supports building/watching/running xcode products, simulators, build/runtime, as well as some lsp support like auto-completion and code navigation.

NOTE: Currently, [Xbase] requires and recommends using [xcodegen] as a way to configure and generate xcode projects without interacting with Xcode.

Checkout [Milestones](https://github.com/tami5/xbase/milestones) for planned future releases and features and [CONTRIBUTING](./CONTRIBUTING.md) if you would like to contribute to the betterment of this endeavor


## 🌝 Motivation

After purchasing a MacBook, I decided to get into iOS/macOS applications development. Though, coming from vim/shell development experience and being heavily keyboard driven, I could not handle the switch to closed sourced, opinionated, mouse-driven development environment. I tried workarounds such as [XVim2] and builtin vim emulator but still I'd catch myself often looking for my mouse.

Being a long time vim user and having previously develop few lua/nvim plugins, I decided to take sometime to invest in both simplify my development workflow for developing `xOS`stuff  and enrich my experience with [rust].

## Table of Content

- [🌟 Features](#-features)
  - [Auto-Completion and Code navigation](#auto-completion-and-code-navigation)
  - [Multi-nvim instance support](#multi-nvim-instance-support)
  - [Auto-start/stop main background daemon](#auto-startstop-main-background-daemon)
  - [Multi Target/Project Support](#multi-targetproject-support)
  - [Simulator Support](#simulator-support)
  - [Logging](#logging)
  - [Statusline Support](#statusline-support)
  - [Zero Footprint](#zero-footprint)
- [🎮 Usage](#-usage)
- [🛠 Requirements](#-requirements)
- [🦾 Installation](#-installation)
- [⚙️ Defaults](#-defaults)
- [🩺 Debugging](#-debugging)
- [🎥 Preview](#-preview)

## 🌟 Features

#### Auto-Completion and Code navigation

> [sourcekit-lsp] doesn't support auto-indexing. Therefore, [xbase] includes a custom build server that auto-generate compiled command on directory changes (i.e. file removed/added).

#### Multi-nvim instance support

> Thanks to having a single daemon running and simple client/server architecture, users can have and use multiple nvim instance running without process duplications and shared state. E.g. stop watch service ran from a nvim instance in another instance.

#### Auto-start/stop main background daemon

> No need to manual start/stop daemon background. The daemon will auto start/stop based on
connect neovim (client instances)

#### Multi Target/Project Support
> Users can develop across different projects and build/run on root directory changes or once.
`*`

#### Simulator Support
> Pick a simulator devices relative to target platform and run the target once or per file changes. If the simulator isn't running then it will auto launch it.

#### Logging
> real-time logging of print statements and build logs. This feature depends partially on [xcodebuild] crate which is still work-in-progress and might not print errors. Currently logs are specific to the requested client, if you find yourself needing shared logs, open an issue.

#### Statusline Support
> Global variable to update statusline with build/run commands + [feline] provider. Other
statusline plugins support are welcomed.

#### Zero Footprint

> light resource use. I've been using xbase for a while now, usually 0.0% cpu and 0.1% memory.


## 🎮 Usage

TLDR:
- Install plugin
- run `require'xbase'.setup({ --[[ see default configuration ]]  })`
- Open xcode codebase with project.yml
- Start coding
- Use available actions which can be configure with shortcuts bellow

Once you start a neovim instance with a root containing `project.yml`, the daemon server will auto-start if no instance is running, and register the project once for recompile-watch, accepting requests from clients. Currently, you should relay on pickers as means to interact with your daemon.

## 🛠 Requirements

- [neovim] v0.7.0 or nightly
- [rust] 1.60.0 or up (see [rust getting started])
- [telescope.nvim]
- [plenery.nvim]

## 🦾 Installation

To install xbase on your system you need run `make install`. This will run `cargo build
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
[Xbase]: https://github.com/tami5/xbase
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
[plenery.nvim]: https://github.com/nvim-lua/plenary.nvim
[neovim]: https://github.com/neovim/neovim
