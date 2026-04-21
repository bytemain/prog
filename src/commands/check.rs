use crate::context::core::Context;
use crate::context::database::models::Repo;
use crate::helpers::colors::Colorize;
use crate::helpers::git::{RepoStatus, get_repo_status};
use rayon::prelude::*;
use serde::Serialize;

#[derive(Debug)]
struct CheckResult {
    repo: Repo,
    status: Option<RepoStatus>,
}

#[derive(Serialize)]
struct JsonStatus<'a> {
    #[serde(flatten)]
    inner: &'a RepoStatus,
    dirty: bool,
    unpushed: bool,
    no_upstream: bool,
}

#[derive(Serialize)]
struct JsonEntry<'a> {
    path: &'a str,
    remote_url: &'a str,
    status: Option<JsonStatus<'a>>,
}

fn collect_results(c: &mut Context) -> Vec<CheckResult> {
    c.auto_sync_silent();
    let items = c.database_mut().get_all_items();
    items
        .into_par_iter()
        .map(|repo| {
            let status = get_repo_status(&repo.full_path);
            CheckResult { repo, status }
        })
        .collect()
}

fn print_group(title: &str, mut entries: Vec<(String, String)>) {
    if entries.is_empty() {
        return;
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    println!("{} ({})", title, entries.len());
    let max_path = entries.iter().map(|(p, _)| p.len()).max().unwrap_or(0);
    for (path, detail) in entries {
        if detail.is_empty() {
            println!("  {}", path);
        } else {
            println!("  {:width$}  {}", path, detail, width = max_path);
        }
    }
    println!();
}

/// Run the check command. Returns the number of repositories with issues
/// (dirty, unpushed, no-upstream, detached, or unreadable).
pub fn run(c: &mut Context, dirty_only: bool, json: bool) -> usize {
    let results = collect_results(c);

    if json {
        return print_json(&results, dirty_only);
    }

    let mut dirty: Vec<(String, String)> = Vec::new();
    let mut unpushed: Vec<(String, String)> = Vec::new();
    let mut no_upstream: Vec<(String, String)> = Vec::new();
    let mut detached: Vec<(String, String)> = Vec::new();
    let mut errored: Vec<(String, String)> = Vec::new();

    for r in &results {
        let path = r.repo.full_path.clone();
        let status = match &r.status {
            Some(s) => s,
            None => {
                errored.push((path, String::new()));
                continue;
            }
        };

        if status.is_dirty() {
            let mut parts = Vec::new();
            if status.modified > 0 {
                parts.push(format!("M:{}", status.modified));
            }
            if status.untracked > 0 {
                parts.push(format!("??:{}", status.untracked));
            }
            if status.conflicted > 0 {
                parts.push(format!("U:{}", status.conflicted));
            }
            dirty.push((path.clone(), parts.join(" ")));
        }

        if dirty_only {
            continue;
        }

        if status.is_unpushed() {
            let detail = match &status.upstream {
                Some(up) => format!("ahead {} [{} -> {}]", status.ahead, status.branch, up),
                None => format!("ahead {} [{}]", status.ahead, status.branch),
            };
            unpushed.push((path.clone(), detail));
        }
        if status.is_no_upstream() {
            no_upstream.push((path.clone(), format!("branch: {}", status.branch)));
        }
        if status.detached {
            detached.push((path.clone(), String::from("detached HEAD")));
        }
    }

    let issue_count =
        dirty.len() + unpushed.len() + no_upstream.len() + detached.len() + errored.len();

    print_group(&"⚠ Dirty".to_string().red().to_string(), dirty);
    if !dirty_only {
        print_group(&"⬆ Unpushed".to_string().yellow().to_string(), unpushed);
        print_group(&"⚑ No upstream".to_string().yellow().to_string(), no_upstream);
        print_group(&"⎇ Detached HEAD".to_string().yellow().to_string(), detached);
    }
    print_group(&"✗ Unreadable".to_string().red().to_string(), errored);

    let total = results.len();
    if issue_count == 0 {
        println!("{}", format!("All {} repositories are clean and synced.", total).green());
    } else {
        println!("{}", format!("{} of {} repositories need attention.", issue_count, total).red());
    }

    issue_count
}

fn print_json(results: &[CheckResult], dirty_only: bool) -> usize {
    let mut issue_count = 0;
    let mut entries: Vec<JsonEntry> = Vec::new();
    for r in results {
        let (status_obj, has_issue) = match &r.status {
            Some(s) => {
                let dirty = s.is_dirty();
                let issue = if dirty_only {
                    dirty
                } else {
                    dirty || s.is_unpushed() || s.is_no_upstream() || s.detached
                };
                (
                    Some(JsonStatus {
                        inner: s,
                        dirty,
                        unpushed: s.is_unpushed(),
                        no_upstream: s.is_no_upstream(),
                    }),
                    issue,
                )
            }
            None => (None, true),
        };

        if dirty_only && !has_issue {
            continue;
        }
        if has_issue {
            issue_count += 1;
        }

        entries.push(JsonEntry {
            path: &r.repo.full_path,
            remote_url: &r.repo.remote_url,
            status: status_obj,
        });
    }
    match serde_json::to_string(&entries) {
        Ok(s) => println!("{}", s),
        Err(e) => eprintln!("Failed to serialize JSON: {}", e),
    }
    issue_count
}
