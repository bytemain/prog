use crate::context::core::Context;
use crate::context::database::models::Repo;
use crate::helpers::colors::Colorize;
use std::collections::HashMap;

pub fn run(c: &mut Context) {
    c.auto_sync_silent();

    let items = c.database_mut().get_all_items();
    // Group by base_dir, then by host
    let mut grouped_by_base_dir: HashMap<String, HashMap<String, Vec<Repo>>> = HashMap::new();

    for item in items {
        // item is Repo
        grouped_by_base_dir
            .entry(item.base_dir.clone())
            .or_default()
            .entry(item.host.clone())
            .or_default()
            .push(item); // Push the whole Repo object
    }

    // Sort hosts alphabetically
    let mut base_dirs: Vec<_> = grouped_by_base_dir.keys().cloned().collect();
    base_dirs.sort();

    for base_dir in base_dirs {
        if let Some(workspaces_in_host) = grouped_by_base_dir.get(&base_dir) {
            println!("{}", base_dir.green()); // Print host name in green

            let mut workspace_keys: Vec<_> = workspaces_in_host.keys().cloned().collect();
            workspace_keys.sort();

            for ws_key in workspace_keys {
                if let Some(repo_items_in_ws) = workspaces_in_host.get(&ws_key) {
                    if !ws_key.is_empty() {
                        println!("  {}", ws_key.as_str().blue());
                    }

                    let mut sorted_repo_items = repo_items_in_ws.clone();
                    sorted_repo_items.sort_by(|a, b| a.full_path.cmp(&b.full_path));

                    for repo_item in sorted_repo_items {
                        let indent = if ws_key.is_empty() { "  " } else { "    " };
                        let path_to_display = repo_item.full_path.clone();
                        println!("{}{}", indent, path_to_display);
                    }
                }
            }
        }
    }
}
