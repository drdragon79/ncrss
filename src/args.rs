use clap::Parser;

#[derive(Debug, Parser)]
#[command(about, version)]
pub struct Args {
    /// Specify the port to listen on
    #[arg(short)]
    pub l: u32,
}