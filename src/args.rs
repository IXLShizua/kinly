use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "kinly", version)]
pub struct Args {
    #[arg(
        default_value = "config.json",
        id = "path",
        long = "config",
        help = "Path to config file (relative/absolute)"
    )]
    pub config_path: PathBuf,

    #[arg(
        default_value = "data",
        id = "data",
        long = "data-dir",
        help = "Path to data directory (relative/absolute)"
    )]
    pub data_dir: PathBuf,
}
