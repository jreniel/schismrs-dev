use clap::Parser;
use clap::ValueEnum;
use std::process::ExitCode;
use storm_events::atcf::ATCFFileDeck;
use storm_events::storm_event::StormEventBuilder;

#[derive(Parser, Debug)]
#[command(author, about, long_about = None)]
// #[command(version = VERSION)]
struct Cli {
    storm_id: String,
    #[clap(short, long)]
    file_deck: Option<FileDeckKind>,
}

#[derive(ValueEnum, Clone, Debug)]
enum FileDeckKind {
    ADVISORY,
    BEST,
    FIXED,
}

fn entrypoint() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let storm_event = StormEventBuilder::default()
        // .file_deck(ATCFFileDeck::BEST)
        .storm_id(&cli.storm_id)
        .build()?;
    dbg!(storm_event);
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
