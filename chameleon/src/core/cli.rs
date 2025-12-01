use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, default_value = "chameleon.toml")]
    pub config_path: String,

    #[arg(short, long, default_value = "style.css")]
    pub style_path: String,
}
