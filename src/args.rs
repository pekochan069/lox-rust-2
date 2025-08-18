use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, default_value_t = log::LevelFilter::Info)]
    pub log_level: log::LevelFilter,

    #[arg(short, long, default_value_t = false)]
    pub disassemble: bool,

    pub source: Option<String>,
}
