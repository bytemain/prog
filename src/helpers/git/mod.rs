mod branch;
mod clone;
mod url;

pub use branch::get_branch;
pub use clone::clone;
pub use url::get_remote_url;
pub use url::remote_url_is_valid;
