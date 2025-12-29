use std::path::PathBuf;

use clap::{Command, CommandFactory, Parser, Subcommand};
use clap_complete::{Shell, generate};
use std::io::{self, Write};

use crate::{commands, helpers::template::render_template};

#[derive(Subcommand, Debug)]
pub enum ECommands {
    #[command(about = "Add a new repository")]
    Add {
        url: String,
        #[arg(allow_hyphen_values = true)]
        rest: Vec<String>,
    },
    #[command(about = "Find a repository by keyword")]
    Find {
        keyword: String,
        #[arg(short = 'q', long = "query", help = "Only query result")]
        query: bool,
    },
    #[command(about = "Sync repositories")]
    Sync,
    #[command(about = "Activate shell")]
    Shell {
        shell: Shell,
    },
    #[command(about = "Import repositories from a path")]
    Import {
        path: PathBuf,
    },
    #[command(about = "Remove a repository by path")]
    Remove {
        path: PathBuf,
        #[arg(short = 'y', long = "yes", help = "Skip confirmation prompt")]
        yes: bool,
    },
    #[command(about = "Clean up repositories")]
    Clean {
        #[arg(short = 'y', long = "yes", help = "Skip confirmation prompt")]
        yes: bool,
    },
    #[command(about = "List all repositories")]
    List,
    Tmp(commands::tmp::TmpArgs),
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, allow_external_subcommands = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<ECommands>,
}

trait ShellScriptName {
    /// Returns the content of the corresponding shell integration script
    fn integration_script(&self) -> &'static str;
}

impl ShellScriptName for Shell {
    fn integration_script(&self) -> &'static str {
        match self {
            Shell::Bash => include_str!("shell-integrations/bash.sh"),
            Shell::PowerShell => include_str!("shell-integrations/powershell.ps1"),
            Shell::Zsh => include_str!("shell-integrations/zsh.sh"),
            _ => "",
        }
    }
}

impl Cli {
    pub fn new() -> Self {
        Cli::parse()
    }
    pub fn activate(shell: Shell) {
        let mut cmd = Cli::command();
        let bin_name = &cmd.get_name().to_string();

        // get all commands and make a vec
        let mut commands = Vec::new();
        for subcommand in cmd.get_subcommands() {
            commands.push(subcommand.get_name());
        }
        // add builtin help
        commands.push("help");

        // transform commands to if check
        // [[ "$1" = commands.0 ]] || [[ "$1" = commands.1 ]] ||...
        let if_check_statement = match shell {
            Shell::PowerShell => {
                // In PowerShell, test first arg against known subcommands or hyphen options
                let mut checks = Vec::new();
                for command in &commands {
                    checks.push(format!("$args[0] -eq '{}'", command));
                }
                checks.push(String::from("$args[0].StartsWith('-')"));
                format!("($args.Count -ge 1) -and ({})", checks.join(" -or "))
            }
            _ => {
                // Bash/Zsh style conditions
                let mut if_check = vec![];
                for command in &commands {
                    if_check.push(format!("[[ \"$1\" = \"{}\" ]]", command));
                }
                // Detect hyphen-prefixed options (e.g., --help, -h) in bash/zsh
                if_check.push(String::from("[[ \"$1\" == -* ]]"));
                if_check.join(" || ")
            }
        };

        let command = "p";

        let script_content = shell.integration_script();

        if script_content.is_empty() {
            eprintln!("Shell integration for {:?} is not supported yet.", shell);
            std::process::exit(1);
        }

        let context = crate::collection! {
            String::from("if_check_statement") => if_check_statement,
            String::from("command") => String::from(command),
        };

        let text = render_template(String::from(script_content), &context);

        // Collect all output in a buffer to ensure it can be properly handled
        // by PowerShell's Invoke-Expression with $() command substitution
        let mut buffer = Vec::new();
        generate(shell, &mut cmd, bin_name, &mut buffer);
        generate(shell, &mut cmd, command, &mut buffer);
        buffer.write_all(text.as_bytes()).expect("Could not write to buffer");

        // For PowerShell, add semicolons after statements to ensure the script works
        // when used with Invoke-Expression "$(prog shell powershell)"
        // This handles the case where PowerShell's $() joins lines with spaces in string context
        if matches!(shell, Shell::PowerShell) {
            let script = String::from_utf8(buffer).expect("Invalid UTF-8 in generated script");
            let mut in_param_block = false;
            let mut paren_depth = 0;
            let mut brace_depth = 0;
            let mut seen_using_namespace = std::collections::HashSet::new();
            
            let lines: Vec<&str> = script.lines().collect();
            let processed = lines
                .iter()
                .enumerate()
                .filter_map(|(idx, line)| {
                    let trimmed = line.trim();

                    // Skip empty lines or lines that are only whitespace
                    if trimmed.is_empty() {
                        return Some(line.to_string());
                    }

                    // Filter out duplicate 'using namespace' statements
                    // PowerShell requires all 'using' statements to appear before other statements
                    // clap_complete generates them multiple times (once per command), causing errors
                    if trimmed.starts_with("using namespace ") {
                        if !seen_using_namespace.insert(trimmed.to_string()) {
                            // Skip duplicate using namespace statements
                            return None;
                        }
                    }

                    // Track if we're entering or leaving a param() block
                    if trimmed.starts_with("param(") || trimmed.contains(" param(") {
                        in_param_block = true;
                        // Reset paren_depth because param() is always at the start of a function/scriptblock
                        // and there should be no unclosed parens from previous lines at this point
                        paren_depth = 0;
                    }
                    
                    if in_param_block {
                        // Track parentheses depth within param block
                        paren_depth += trimmed.matches('(').count() as i32;
                        paren_depth -= trimmed.matches(')').count() as i32;
                        
                        // Exit param block when we close all parens
                        if paren_depth <= 0 {
                            in_param_block = false;
                        }
                        
                        // Don't add semicolons inside param blocks
                        return Some(line.to_string());
                    }

                    // Check if the next line starts with a closing paren or brace
                    // or a continuation operator - these indicate the current line is part of
                    // a multi-line expression and shouldn't have a semicolon
                    let next_line_continues = if idx + 1 < lines.len() {
                        let next_trimmed = lines[idx + 1].trim();
                        // Don't add semicolon if next line starts with closing paren or operators
                        // But DO add semicolon if next line is just '}' or '};' and we're inside a block
                        let is_closing_brace_only = next_trimmed == "}" || next_trimmed == "};";
                        // Don't add semicolon if current line ends with } and next line is catch/finally/elseif/else
                        let is_control_flow_continuation = trimmed.ends_with('}') && (
                            next_trimmed.starts_with("catch")
                            || next_trimmed.starts_with("finally")
                            || next_trimmed.starts_with("elseif")
                            || next_trimmed.starts_with("else")
                        );
                        is_control_flow_continuation || (!is_closing_brace_only && (
                            next_trimmed.starts_with(')')
                            || next_trimmed.starts_with('}')
                            || next_trimmed.starts_with("-or")
                            || next_trimmed.starts_with("-and")
                        ))
                    } else {
                        false
                    };

                    // Track brace depth (though not currently used, kept for future enhancements)
                    brace_depth += trimmed.matches('{').count() as i32;
                    brace_depth -= trimmed.matches('}').count() as i32;

                    // Add semicolon to every line except:
                    // - Lines that end with continuation characters
                    // - Comment lines (they don't need semicolons)
                    // - Lines where the next line continues the expression
                    let needs_semicolon = !next_line_continues
                        && !trimmed.starts_with('#')
                        && !trimmed.ends_with('{')
                        && !trimmed.ends_with('(')
                        && !trimmed.ends_with(',')
                        && !trimmed.ends_with('|')
                        && !trimmed.ends_with('\\')
                        && !trimmed.ends_with("-or")
                        && !trimmed.ends_with("-and");

                    if needs_semicolon {
                        Some(format!("{};", line))
                    } else {
                        Some(line.to_string())
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");
            io::stdout().write_all(processed.as_bytes()).expect("Could not write to stdout");
        } else {
            io::stdout().write_all(&buffer).expect("Could not write to stdout");
        }
    }

    pub fn show_help() {
        let mut cmd = Cli::command();
        cmd.print_help().expect("Could not print help");
        std::process::exit(1);
    }

    pub fn get_subcommand(sub_cmd: &str) -> Option<Command> {
        let mut cmd = Cli::command();

        for subcommand in cmd.get_subcommands_mut() {
            // Use get_subcommands_mut() if you need to modify, or clone directly if Command is Clone
            if subcommand.get_name() == sub_cmd {
                return Some(subcommand.clone()); // Clone the subcommand
            }
        }
        None
    }
}
