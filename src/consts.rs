use std::path::PathBuf;

use once_cell::sync::Lazy;

pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");
pub static SELF_FILE_PATH: Lazy<PathBuf> =
    Lazy::new(|| std::env::current_exe().unwrap_or_default());
pub static CONFIG_FILE_PATH: Lazy<PathBuf> = Lazy::new(|| {
    dirs::home_dir()
        .unwrap_or_default()
        .join(format!(".config/{PKG_NAME}.lua"))
});
pub static CACHE_DIR_PATH: Lazy<PathBuf> = Lazy::new(|| {
    dirs::home_dir()
        .unwrap_or_default()
        .join(format!(".cache/{PKG_NAME}"))
});
