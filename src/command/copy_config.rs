use crate::config::Config;

pub fn copy_config() -> anyhow::Result<()> {
    Config::copy_default()
}
