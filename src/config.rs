use serde::Deserialize;
use std::error::Error;
use std::fs;
use std::process::Command;
use tracing::{debug, error, info};

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub blocks: Vec<DisplayBlock>,
    pub hardware: HardwareConfig, // Add hardware settings
}

#[derive(Debug, Deserialize, Clone)]
pub struct HardwareConfig {
    pub i2c_bus: String,
    pub chip: String,
    pub clk: u32,
    pub dt: u32,
    pub sw: u32,
}

#[derive(Debug, Deserialize, Clone)] // Added Clone
pub struct DisplayBlock {
    pub top_line: String,
    pub second_line: String,
    pub button_enabled: bool,
    pub function_to_run: String,
    pub refresh_interval: u64, // Time in seconds (0 = no refresh)
}

impl DisplayBlock {
    pub fn get_top_line(&self) -> String {
        if self.top_line.starts_with("!") {
            run_command(&self.top_line[1..])
        } else {
            self.top_line.clone()
        }
    }

    pub fn get_second_line(&self) -> String {
        if self.second_line.starts_with("!") {
            run_command(&self.second_line[1..])
        } else {
            self.second_line.clone()
        }
    }
}

pub fn run_command(cmd: &str) -> String {
    info!("Running command: {}", cmd);

    // Try running the command directly
    let direct_output = Command::new(cmd)
        .output()
        .ok()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string());

    if let Some(output) = direct_output {
        if !output.is_empty() {
            info!("Command output: {}", output);
            return output;
        }
    }

    // Fall back to running it with `sh -c`
    let shell_output = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .ok()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string());

    if let Some(output) = &shell_output {
        info!("Shell command output: {}", output);
    } else {
        error!("Failed to run command: {}", cmd);
    }

    shell_output.unwrap_or_else(|| format!("Failed to run: {}", cmd))
}

pub fn load_config(path: &str) -> Result<Config, Box<dyn Error>> {
    info!("Loading config from path: {}", path);

    let content = fs::read_to_string(path)?;
    info!("Config file read successfully");

    let config: Config = toml::from_str(&content)?;
    info!("Config file parsed successfully");

    debug!("Loaded config: {:?}", config);

    Ok(config)
}
