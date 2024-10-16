use sloughi::Sloughi;

fn main() {
    let _ = Sloughi::new()
        .custom_path(".husky")
        .ignore_env("CI") // Ignore setup when `CI` environment variable is set (like in CircleCI ..etc)
        .ignore_env("GITHUB_ACTIONS") // Do not run in GitHub Actions as well
        .install();
}
