use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use clap::{arg, command, value_parser, Arg, ArgAction, Command};
use clap::{Parser, ValueEnum};
use dateparser;
use schismrs_bctides::bctides::BctidesBuilder;
use schismrs_hgrid::Hgrid;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser, Debug)]
struct Cli {
    hgrid_path: PathBuf,
    #[clap(value_parser = dateparser::parse, help="See https://docs.rs/dateparser/latest/dateparser/#accepted-date-formats for a list of accepted input formats.")]
    start_date: DateTime<Utc>,
    /// aliases: tip-dp, tip_dp, cutoff_depth, cutoff-depth, tpcd
    #[clap(short, long, aliases = &["tip-dp", "tip_dp", "cutoff_depth", "cutoff-depth", "tpcd"])]
    tidal_potential_cutoff_depth: f64,
}

// #[derive(ValueEnum, Clone, Debug)]
// enum ConstituentsKind {
//     /// major
//     Q1,
//     O1,
//     P1,
//     K1,
//     N2,
//     M2,
//     S2,
//     K2,
//     /// minor
//     Mm,
//     Mf,
//     M4,
//     MN4,
//     MS4,
//     _2N2,
// }

fn entrypoint() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let hgrid = Hgrid::try_from(&cli.hgrid_path)?;
    let boundaries = hgrid
        .boundaries()
        .ok_or_else(|| anyhow!("The mesh file has no defined boundaries."))?;
    let open_boundaries = boundaries
        .open()
        .ok_or_else(|| anyhow!("The mesh has no defined open boundaries."))?;
    let mut runtime_cli = command!();
    for (i, bnd) in open_boundaries.iter().enumerate() {
        dbg!(i);
        dbg!(bnd);
        // Dynamically create and add the argument
        let arg_name = format!("elevation-{}", i + 1).as_str();
        let arg = Arg::new(arg_name) // Use the dynamic name here
            .long(arg_name) // Set the long name without the leading '--'
            .required(false); // Set whether the argument is required

        // runtime_cli = runtime_cli.arg(arg);
        // runtime_cli
        //     .arg(arg!(format!("--elevation-{}", i + 1)))
        //     .required(false);
    }
    let matches = runtime_cli.get_matches();
    dbg!(matches);

    let mut builder = BctidesBuilder::default();
    let bctides = builder
        .hgrid(&hgrid)
        .start_date(&cli.start_date)
        .tidal_potential_cutoff_depth(&cli.tidal_potential_cutoff_depth)
        // .constituents()
        .build()?;
    println!("{}", &bctides);
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
