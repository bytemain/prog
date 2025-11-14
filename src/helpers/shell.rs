// This module previously contained a shell::run() function that used bash -c
// to execute commands. It has been removed because:
// 1. It caused Windows path handling issues (backslashes were corrupted)
// 2. It hardcoded bash which doesn't exist by default on Windows
// 3. It's no longer used anywhere in the codebase
//
// Git operations now use std::process::Command directly for proper cross-platform support.
