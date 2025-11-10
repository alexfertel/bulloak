use anyhow::Result;
pub mod config;
pub use config::Config;

pub fn scaffold(_text: &String, _cfg: &Config) -> Result<String> {
    Ok("".to_string())
}
