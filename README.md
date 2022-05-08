# XcodeBase.nvim

Like Xcode but for neovim.

## Features 

- Auto-generate compiled command on directory changes (i.e. file removed/added).
- Auto-start daemon for processing commands.
- Completion and code navigation using custom build server.
- Command plate:
   - build <Target>:       Build the target.
   - archive <Scheme>:     Archive a scheme. (\*)
   - test <Scheme>:        Test a scheme. (\*)
   - install <Target>:     Build the target and install it to (DSTROOT). (\*)

