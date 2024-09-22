use sloughi::Sloughi;

fn main() {
    let _ = Sloughi::new()
        .custom_path(".git_hooks") // Choose a custom Git hooks relative path (default is ".sloughi")
        .ignore_env("CI") // Ignore setup when `CI` environment variable is set (like in CircleCI ..etc)
        .ignore_env("GITHUB_ACTIONS") // Do not run in GitHub Actions as well
        .install();
}
