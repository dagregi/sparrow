use clap::Parser;

use crate::config::{get_config_dir, get_data_dir};

const DEFAULT_URL: &str = "http://localhost:9091/transmission/rpc";

#[derive(Parser, Debug)]
#[command(author, version = version(), about)]
/// TUI for transmission remote
pub struct Cli {
    /// RPC url
    #[arg(
        short,
        long,
        value_name = "URL",
        value_parser = validate_url,
        default_value = DEFAULT_URL
    )]
    pub url: String,
    /// Set username for authentication
    #[arg(long, value_name = "USERNAME")]
    pub username: Option<String>,
    /// Set password for authentication
    #[arg(long, value_name = "PASSWORD")]
    pub password: Option<String>,
    /// Tick rate, i.e. number of ticks per second
    #[arg(short, long, value_name = "FLOAT", default_value_t = 4.0)]
    pub tick_rate: f64,

    /// Frame rate, i.e. number of frames per second
    #[arg(short, long, value_name = "FLOAT", default_value_t = 30.0)]
    pub frame_rate: f64,
}

fn validate_url(url: &str) -> Result<String, String> {
    let components: Vec<&str> = url.split("://").collect();
    if components.len() != 2 {
        return Err("Invalid URL: URL should have a scheme.".to_string());
    }

    let scheme = components[0];
    if scheme != "http" && scheme != "https" {
        return Err("Invalid scheme: Scheme should be either 'http' or 'https'.".to_string());
    }

    Ok(url.to_string())
}

const VERSION_MESSAGE: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    "-",
    env!("VERGEN_GIT_DESCRIBE"),
    " (",
    env!("VERGEN_BUILD_DATE"),
    ")"
);

pub fn version() -> String {
    let config_dir_path = get_config_dir().display().to_string();
    let data_dir_path = get_data_dir().display().to_string();

    format!(
        "\
{VERSION_MESSAGE}

Config directory: {config_dir_path}
Data directory: {data_dir_path}"
    )
}
