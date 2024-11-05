use clipboard_rs::{Clipboard, ClipboardContent, ClipboardContext};
use crossterm::style::Stylize;

pub fn copy_path(path: &str) {
    let ctx = ClipboardContext::new().unwrap();
    ctx.set(vec![ClipboardContent::Text(format!("cd {}", path))])
        .expect("Failed to set clipboard content");

    println!("{}", "📋 Copied to clipboard, you can paste it now.".green());
}
