use std::process::{Command, Stdio};
use std::io::Write;
use crate::helpers::colors::Colorize;
use std::env;

pub fn copy_path(path: &str) {
    let content = format!("cd {}", path);
    let success = if cfg!(target_os = "windows") {
        copy_to_clipboard_windows(&content)
    } else if cfg!(target_os = "macos") {
        copy_to_clipboard_macos(&content)
    } else if cfg!(target_os = "linux") {
        copy_to_clipboard_linux(&content)
    } else {
        // For other systems, we show an error message
        eprintln!("Clipboard functionality is not supported on this platform.");
        false
    };

    if success {
        println!("{}", "📋 Copied to clipboard, you can paste it now.".green());
    } else {
        eprintln!("Failed to copy to clipboard.");
    }
}

#[cfg(target_os = "windows")]
fn copy_to_clipboard_windows(content: &str) -> bool {
    // On Windows, use PowerShell's Set-Clipboard command
    let mut child = match Command::new("powershell")
        .arg("-Command")
        .arg("Set-Clipboard -Value $input")
        .stdin(Stdio::piped())
        .spawn() {
            Ok(child) => child,
            Err(_) => return false,
        };

    // Write content to PowerShell's stdin
    if let Some(stdin) = child.stdin.as_mut() {
        if stdin.write_all(content.as_bytes()).is_err() {
            return false;
        }
    } else {
        return false;
    }

    // Wait for the command to complete
    match child.wait() {
        Ok(status) => status.success(),
        Err(_) => false,
    }
}

#[cfg(target_os = "macos")]
fn copy_to_clipboard_macos(content: &str) -> bool {
    // On macOS, use the pbcopy command
    let mut child = match Command::new("pbcopy")
        .stdin(Stdio::piped())
        .spawn() {
            Ok(child) => child,
            Err(_) => return false,
        };

    // Write content to pbcopy's stdin
    if let Some(stdin) = child.stdin.as_mut() {
        if stdin.write_all(content.as_bytes()).is_err() {
            return false;
        }
    } else {
        return false;
    }

    // Wait for the command to complete
    match child.wait() {
        Ok(status) => status.success(),
        Err(_) => false,
    }
}

#[cfg(target_os = "linux")]
fn copy_to_clipboard_linux(content: &str) -> bool {
    // Try multiple Linux clipboard utilities
    
    // 1. Try xclip (for X11 environments)
    if let Ok(mut child) = Command::new("xclip")
        .arg("-selection")
        .arg("clipboard")
        .stdin(Stdio::piped())
        .spawn() {
        
        if let Some(stdin) = child.stdin.as_mut() {
            if stdin.write_all(content.as_bytes()).is_ok() {
                return match child.wait() {
                    Ok(status) => status.success(),
                    Err(_) => false,
                };
            }
        }
    }
    
    // 2. Try xsel (alternative for X11 environments)
    if let Ok(mut child) = Command::new("xsel")
        .arg("--clipboard")
        .arg("--input")
        .stdin(Stdio::piped())
        .spawn() {
        
        if let Some(stdin) = child.stdin.as_mut() {
            if stdin.write_all(content.as_bytes()).is_ok() {
                return match child.wait() {
                    Ok(status) => status.success(),
                    Err(_) => false,
                };
            }
        }
    }
    
    // 3. Try wl-copy (for Wayland environments)
    if let Ok(mut child) = Command::new("wl-copy")
        .stdin(Stdio::piped())
        .spawn() {
        
        if let Some(stdin) = child.stdin.as_mut() {
            if stdin.write_all(content.as_bytes()).is_ok() {
                return match child.wait() {
                    Ok(status) => status.success(),
                    Err(_) => false,
                };
            }
        }
    }
    
    // All attempts failed
    eprintln!("No clipboard utility found. Please install xclip, xsel, or wl-copy.");
    false
}

// Provide empty implementations for other platforms to avoid compilation errors
#[cfg(not(target_os = "windows"))]
fn copy_to_clipboard_windows(_content: &str) -> bool {
    false
}

#[cfg(not(target_os = "macos"))]
fn copy_to_clipboard_macos(_content: &str) -> bool {
    false
}

#[cfg(not(target_os = "linux"))]
fn copy_to_clipboard_linux(_content: &str) -> bool {
    false
}
