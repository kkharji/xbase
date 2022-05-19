if exists("b:current_syntax")
  finish
endif

let s:cpo_save = &cpo
set cpo&vim


syn match   XbaseOperations   "\(Executing\|Compiling\|Generating\|Processing\|Emitting\|Compiling\|Copying\|Validating\|Signing\|Linking\|RegisterLaunchServices\)"
syn match   XbaseEntitlement  "Entitlement"
syn region  XbaseScope        display oneline start='^\[' end='\]'
syn match   XbaseLogError     "^\(\[Error\]\)"
syn match   XbaseLogWarn      "^\(\[Warning\]\)"
syn match   XbaseLogSuccess   "^\(\[Succeed\]\)"
syn match   XbaseLogDone      "^\(\[Done\]\)"
syn match   XbaseLogPackage   "^\(\[Package\]\)"
syn match   XbaseLogLaunched  "^\(\[Launched\]\)"
syn match   XbaseLogOutput    "^\(\[Output\]\)"
syn match   XbaseLogOutput    "^\(\[Exit\]\)"
syn match   XbaseRunning    "^\(\[Running\]\)"
syn match   XbaseTarget       "`\(\w.*\)`"
syn match   XbaseFilePath     "`\(\/.*\)`" 
syn region  XbaseSep          display oneline start='-' end='-$'

hi def link XbaseScope         Label
hi def link XbaseLogSuccess    healthSuccess
hi def link XbaseOperations    Function
hi def link XbaseEntitlement   Comment
hi def link XbaseLogOutput     Comment
hi def link XbaseRunning     Comment
hi def link XbaseSep           Comment
hi def link XbaseLogPackage    Comment
hi def link XbaseFilePath      String
hi def link XbaseTarget        Label
hi def link XbaseLogError      Error
hi def link XbaseLogWarn       WarningMsg
hi def link XbaseLogLaunched   healthSuccess

syn match HideAa "\`" conceal


let b:current_syntax = "xcodebuildlog"

let &cpo = s:cpo_save
unlet s:cpo_save
