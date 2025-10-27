use std::env;
use std::fmt;

// ANSI color codes
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
#[allow(dead_code)]
const YELLOW: &str = "\x1b[33m";
const BLUE: &str = "\x1b[34m";
const RESET: &str = "\x1b[0m";

// Struct to hold a colored string
pub struct ColoredString {
    content: String,
    color_code: &'static str,
}

// Check if colors should be displayed based on environment variables
fn colors_enabled() -> bool {
    // Check for explicit color disabling variables
    if env::var("NO_COLOR").is_ok() {
        return false;
    }

    if let Ok(color) = env::var("CLICOLOR") {
        if color == "0" {
            return false;
        }
    }

    if let Ok(force) = env::var("CLICOLOR_FORCE") {
        if force == "1" {
            return true;
        }
    }

    // Check for CI environments where colors are often not wanted
    if env::var("CI").is_ok() || env::var("CONTINUOUS_INTEGRATION").is_ok() {
        return false;
    }

    // Check for non-interactive terminals
    if let Ok(term) = env::var("TERM") {
        if term == "dumb" {
            return false;
        }
    }

    // Default to enabling colors
    true
}

impl fmt::Display for ColoredString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if colors_enabled() {
            write!(f, "{}{}{}", self.color_code, self.content, RESET)
        } else {
            write!(f, "{}", self.content)
        }
    }
}

// Extension trait to add color methods to String and &str
pub trait Colorize {
    fn red(self) -> ColoredString;
    fn green(self) -> ColoredString;
    #[allow(dead_code)]
    fn yellow(self) -> ColoredString;
    fn blue(self) -> ColoredString;
}

impl Colorize for String {
    fn red(self) -> ColoredString {
        ColoredString { content: self, color_code: RED }
    }

    fn green(self) -> ColoredString {
        ColoredString { content: self, color_code: GREEN }
    }

    fn yellow(self) -> ColoredString {
        ColoredString { content: self, color_code: YELLOW }
    }

    fn blue(self) -> ColoredString {
        ColoredString { content: self, color_code: BLUE }
    }
}

impl Colorize for &str {
    fn red(self) -> ColoredString {
        ColoredString { content: self.to_string(), color_code: RED }
    }

    fn green(self) -> ColoredString {
        ColoredString { content: self.to_string(), color_code: GREEN }
    }

    fn yellow(self) -> ColoredString {
        ColoredString { content: self.to_string(), color_code: YELLOW }
    }

    fn blue(self) -> ColoredString {
        ColoredString { content: self.to_string(), color_code: BLUE }
    }
}
