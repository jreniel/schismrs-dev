use clap::Parser;
use std::process::ExitCode;
use storm_events::StormEventBuilder;

#[derive(Parser, Debug)]
#[command(author, about, long_about = None)]
// #[command(version = VERSION)]
struct Cli {
    storm_id: String,
}

fn entrypoint() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let storm_event = StormEvent::try_from(cli.storm_id)?;
    Ok(())
}

fn main() -> ExitCode {
    let exit_code = match entrypoint() {
        Err(e) => {
            eprintln!("Error: {}", e);
            return ExitCode::FAILURE;
        }
        Ok(_) => ExitCode::SUCCESS,
    };
    exit_code
}
