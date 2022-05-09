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
   - watch                 Rebuild or Rerun when project files get modified
- Build/Run Watch Service:
  - Support multi nvim instance (one watch instance).
  - Stop watch service from another instance.
  - Helper function to to update status line (\*)
  - Auto open log buffer on error (\*)

