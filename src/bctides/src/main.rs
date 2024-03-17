use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use clap::command;
use clap::Parser;
use schismrs_hgrid::boundaries::OpenBoundaries;
use schismrs_hgrid::Hgrid;
use std::path::PathBuf;
use std::process::ExitCode;
// use dateparser;
// use schismrs_bctides::bctides::BctidesBuilder;
// use getargs::{Arg, Error, Options};

// | Variable     | Type -4, -5 (`uv3D.th`); Nudging                                  | Nudging/Sponge layer near bnd |
// |--------------|-------------------------------------------------------------------|--------------------------------------------|--------------------------------|
// | η            | N/A                                                               | `inu_elev=1`                     |
// | S&T, Tracers | N/A                                                               | `inu_[MOD]=1 or 2`               |
// | u,v          | Relax to `uv3D.th.nc` (2 separate relaxations for in and outflow) | `inu_uv=1`                        |

#[derive(Parser, Debug)]
#[command(
    about,
    version,
    ignore_errors = true,
    after_help = "\x1b[1m\x1b[4mDynamic Arguments:\x1b[0m \
    This program uses the input hgrid to generate additional dynamic arguments which depend on the input hgrid \
    and can be used to assign different forcing types to each boundary.\n  \
    \x1b[1m--<VARIABLE_NAME>-<BOUNDARY_ID>=<IBTYPE>\x1b[0m\n\
    \tFor example: --elevation-1=4 applies elevation type 4 to boundary id 0.\n \
    \x1b[1m--<VARIABLE_NAME>-db-<BOUNDARY_ID>=<DATABASE_NAME>\x1b[0m\n\
    \nBoundary ID's begin at 1 as per the convention of hgrid files.\n\n\
    The following tables describe each variable and associated types:\n\n\
| Variable     | Type 1 (`*.th`)                                                       |
|--------------|-----------------------------------------------------------------------|
| η            | `elev.th`; Time history; uniform along bnd                            |
| S&T, Tracers | `[MOD]_[ID].th`: relax to time history (uniform along bnd for inflow) |
| u,v          | `flux.th`: via discharge ( <0 for inflow!)                            |

| Variable     | Type 2                              |
|--------------|-------------------------------------|
| η            | constant                            |
| S&T, Tracers | Relax to specified value for inflow |
| u,v          | Via discharge (<0 for inflow)       |

| Variable     | Type 3                                  |
|--------------|-----------------------------------------|
| η            | Tidal amp/phases                        |
| S&T, Tracers | Relax to i.c. for inflow                |
| u,v          | Tidal amp/phases for u and v components |


| Variable     | Type 4 (`*[23]D.th`)                                                               |
|--------------|------------------------------------------------------------------------------------|
| η            | `elev2D.th.nc`: time- and space- varying along bnd                                 |
| S&T, Tracers | `[MOD]_3D.th.nc`: relax to time- and space- varying values along bnd during inflow |
| u,v          | `uv3D.th.nc`: time- and space- varying along bnd (in lon/lat for `ics=2`)          |

| Variable     | Type 5                         |
|--------------|--------------------------------|
| η            | `elev2D.th.nc`: sum of 3 and 4 |
| S&T, Tracers | N/A                            |
| u,v          | `uv3D.th.nc`: sum of 3 and 4   |

| Variable     | Type -1           |
|--------------|-------------------|
| η            | Must = 0          |
| S&T, Tracers | N/A               |
| u,v          | Flather (0 for η) |

| Variable     | Type -4, -5 (`uv3D.th`); Nudging                                  |
|--------------|-------------------------------------------------------------------|
| η            | N/A                                                               |
| S&T, Tracers | N/A                                                               |
| u,v          | Relax to `uv3D.th.nc` (2 separate relaxations for in and outflow) |

Variable legend:
 η: elevation
 S: salinity
 T: temperature
 u,v: velocity
"
)]
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
//     _2N2,is
// }
//
//
fn get_boundary_forcing_config(hgrid: &Hgrid) -> Result<BoundaryForcingConfig> {
    let BoundaryForcingConfigBuilder
    Ok(boundary_forcing_config)
}

fn entrypoint() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let hgrid = Hgrid::try_from(&cli.hgrid_path)?;
    // let boundaries = hgrid
    //     .boundaries()
    //     .ok_or_else(|| anyhow!("The mesh file has no defined boundaries."))?;
    // let open_boundaries = boundaries
    //     .open()
    //     .ok_or_else(|| anyhow!("The mesh has no defined open boundaries."))?;
    let boundary_forcing_config = get_boundary_forcing_config(&hgrid)?;
    // // let mut runtime_cli = command!();
    // for (i, bnd) in open_boundaries.iter().enumerate() {
    //     dbg!(i);
    //     dbg!(bnd);
    //     // Dynamically create and add the argument
    //     // let arg_name = format!("elevation-{}", i + 1).as_str();
    //     // let arg = Arg::new(arg_name) // Use the dynamic name here
    //     //     .long(arg_name) // Set the long name without the leading '--'
    //     //     .required(false); // Set whether the argument is required

    //     // runtime_cli = runtime_cli.arg(arg);
    //     // runtime_cli
    //     //     .arg(arg!(format!("--elevation-{}", i + 1)))
    //     //     .required(false);
    // }
    // let matches = runtime_cli.get_matches();
    // dbg!(matches);

    // let mut builder = BctidesBuilder::default();
    // let bctides = builder
    //     .hgrid(&hgrid)
    //     .start_date(&cli.start_date)
    //     .tidal_potential_cutoff_depth(&cli.tidal_potential_cutoff_depth)
    //     // .constituents()
    //     .build()?;
    // println!("{}", &bctides);
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
