use clap::Parser;
use std::process;

mod args;
mod listener;

fn main() {
    // Parse arguments
    let args = args::Args::parse();

    // Start listener
    listener::listen(args)
        .unwrap_or_else(|e| {
            print!("Failed to start lister: {e}");
            process::exit(1);
        });
}
