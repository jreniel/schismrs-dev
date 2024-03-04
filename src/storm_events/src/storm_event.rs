use crate::atcf::ATCFFileDeck;
use chrono::{Datelike, Utc};
use datetime::Year;
use polars::frame::DataFrame;
use polars::prelude::*;
use polars_lazy::prelude::*;
use regex::Regex;
use smartstring::alias::String as SmartString;
use std::io::Cursor;
use std::sync::Arc;
use thiserror::Error;
use url::{ParseError, Url};

#[derive(Debug)]
pub struct StormEvent {
    // name: String,
    // year: Year,
    track: DataFrame,
}

#[derive(Default)]
pub struct StormEventBuilder<'a> {
    file_deck: Option<&'a ATCFFileDeck>,
    storm_id: Option<&'a str>,
}

impl<'a> StormEventBuilder<'a> {
    pub fn build(&self) -> Result<StormEvent, StormEventBuilderError> {
        let storm_id = self.storm_id.ok_or_else(|| {
            StormEventBuilderError::UninitializedFieldError("storm_id".to_string())
        })?;

        let track = self.get_track_from_storm_id(storm_id)?;

        Ok(StormEvent { track })
    }

    pub fn storm_id(&mut self, storm_id: &'a str) -> &mut Self {
        self.storm_id = Some(storm_id);
        self
    }

    pub fn file_deck(&mut self, file_deck: &'a ATCFFileDeck) -> &mut Self {
        self.file_deck = Some(file_deck);
        self
    }
    fn get_track_from_storm_id(&self, storm_id: &str) -> Result<DataFrame, StormEventBuilderError> {
        let nhc_code = match (
            Regex::new(r"^[a-zA-Z]{2}\d{6}$")
                .unwrap()
                .is_match(&storm_id),
            Regex::new(r"^[a-zA-Z]*\d{4}$").unwrap().is_match(&storm_id),
        ) {
            (true, _) => Ok(storm_id.to_string()),
            (_, true) => {
                // this branch handles NameYear format
                let storm_name = &storm_id[..storm_id.len() - 4];
                let year = Year(storm_id[storm_id.len() - 4..].parse::<i64>().map_err(|_| {
                    StormEventBuilderError::NoMatchingPatternForStormID(storm_id.to_string())
                })?);
                let inventory = Self::get_nhc_storm_inventory()?;
                Self::get_nhc_code_from_storm_name_and_year(&inventory, storm_name, &year)
            }
            (_, _) => Err(StormEventBuilderError::NoMatchingPatternForStormID(
                storm_id.to_string(),
            )),
        }?;
        let track = self.get_track_from_nhc_code(&nhc_code)?;
        dbg!(track);
        unimplemented!();
    }

    fn get_track_from_nhc_code(&self, nhc_code: &str) -> Result<DataFrame, StormEventBuilderError> {
        let storm_year = Year(nhc_code[nhc_code.len() - 4..].parse::<i64>().map_err(|_| {
            StormEventBuilderError::NoMatchingPatternForNhcCode(nhc_code.to_string())
        })?);
        let current_year = Year(Utc::now().year() as i64);
        let url = "ftp://ftp.nhc.noaa.gov/atcf";
        let file_deck = self.file_deck.ok_or_else(|| {
            StormEventBuilderError::UninitializedFieldError("file_deck".to_string())
        })?;
        let suffix = match storm_year == current_year {
            true => match file_deck {
                ATCFFileDeck::ADVISORY => {
                    format!("aid_public/a{}.dat.gz", nhc_code.to_lowercase()).to_string()
                }
                ATCFFileDeck::BEST => format!("btk/b{}.dat", nhc_code.to_lowercase()).to_string(),
                ATCFFileDeck::FIXED => format!("fix/f{}.dat", nhc_code.to_lowercase()).to_string(),
            },
            false => match file_deck {
                ATCFFileDeck::ADVISORY => format!(
                    "archive/{}/a{}.dat.gz",
                    storm_year.0,
                    nhc_code.to_lowercase()
                )
                .to_string(),
                ATCFFileDeck::BEST => {
                    format!("archive/{}/b{}.dat", storm_year.0, nhc_code.to_lowercase()).to_string()
                }
                ATCFFileDeck::FIXED => {
                    format!("archive/{}/f{}.dat", storm_year.0, nhc_code.to_lowercase()).to_string()
                }
            },
        };
        let url = Url::parse(&format!("{}/{}", url, suffix)).expect("unreachable");
        dbg!(&url.to_string());
        unimplemented!();

        // Ok(Url::parse(&format!(
        //     "ftp://ftp.nhc.noaa.gov/atcf/{}/",
        //     nhc_dir
        // ))?)
        // let atcf_url_prefix = self.get_atcf_prefix_from_year(&storm_year, &current_year)?;
        // let nhc_dir = match storm_year == current_year {
        //     true => {
        //         let file_deck = self
        //             .file_deck
        //             .ok_or_else(|| StormEventBuilderError::UninitializedFileDeckError)?;
        //         match file_deck {
        //             ATCFFileDeck::ADVISORY => "aid_public".to_string(),
        //             ATCFFileDeck::BEST => "btk".to_string(),
        //             ATCFFileDeck::FIXED => "fix".to_string(),
        //         }
        //     }
        //     false => {
        //         format!("archive/{}", year.0)
        //     }
        // };
        // dbg!(atcf_url_prefix);
        // dbg!(nhc_code);
        // unimplemented!();
        // Ok(track)
    }

    fn get_nhc_code_from_storm_name_and_year(
        inventory: &DataFrame,
        storm_name: &str,
        year: &Year,
    ) -> Result<String, StormEventBuilderError> {
        let some_coll = inventory
            .clone()
            .lazy()
            .filter(
                col("name")
                    .eq(lit(format!("{:>10}", storm_name.to_uppercase())))
                    .and(col("year").eq(lit(year.0))),
            )
            .collect()?;
        if some_coll.height() > 1 {
            return Err(StormEventBuilderError::MultipleMatchingData {
                storm_name: storm_name.to_owned(),
                year: year.0,
            });
        } else if some_coll.height() < 1 {
            return Err(StormEventBuilderError::NoMatchingData {
                storm_name: storm_name.to_owned(),
                year: year.0,
            });
        }

        let nhc_code_column = some_coll.column("nhc_code")?;

        let nhc_code_value = nhc_code_column.get(0);

        let nhc_code = nhc_code_value?.to_string();
        let nhc_code = nhc_code.trim_matches('\"').trim().to_string();
        Ok(nhc_code)
    }
    // fn get_storm_name_and_year_from_nhc_code(
    //     inventory: &DataFrame,
    //     nhc_code: &str,
    // ) -> Result<(String, Year), StormEventBuilderError> {
    //     let some_coll = inventory
    //         .clone()
    //         .lazy()
    //         .filter(col("nhc_code").eq(lit(format!("{:>9}", nhc_code.to_uppercase()))))
    //         .collect()?;
    //     if some_coll.height() > 1 {
    //         return Err(StormEventBuilderError::MultipleMatchingNhcCode(
    //             nhc_code.to_owned(),
    //         ));
    //     } else if some_coll.height() == 0 {
    //         return Err(StormEventBuilderError::NoMatchingNhcCode(
    //             nhc_code.to_owned(),
    //         ));
    //     }
    //     dbg!(some_coll);
    //     unimplemented!();
    //     // Ok(())
    // }
    fn get_nhc_storm_inventory() -> Result<DataFrame, StormEventBuilderError> {
        let url = "https://ftp.nhc.noaa.gov/atcf/index/storm_list.txt";
        let response = reqwest::blocking::get(url)?.text()?;
        let cursor = Cursor::new(response);
        let mut schema = Schema::new();
        schema.with_column(SmartString::from("name"), DataType::String);
        schema.with_column(SmartString::from("basin"), DataType::String);
        schema.with_column(SmartString::from("2"), DataType::String);
        schema.with_column(SmartString::from("3"), DataType::String);
        schema.with_column(SmartString::from("4"), DataType::String);
        schema.with_column(SmartString::from("5"), DataType::String);
        schema.with_column(SmartString::from("6"), DataType::String);
        schema.with_column(SmartString::from("number"), DataType::String);
        schema.with_column(SmartString::from("year"), DataType::Int32);
        schema.with_column(SmartString::from("class"), DataType::String);
        schema.with_column(SmartString::from("10"), DataType::String);
        schema.with_column(SmartString::from("start_date"), DataType::String);
        schema.with_column(SmartString::from("end_date"), DataType::String);
        schema.with_column(SmartString::from("13"), DataType::String);
        schema.with_column(SmartString::from("14"), DataType::String);
        schema.with_column(SmartString::from("15"), DataType::String);
        schema.with_column(SmartString::from("16"), DataType::String);
        schema.with_column(SmartString::from("17"), DataType::String);
        schema.with_column(SmartString::from("source"), DataType::String);
        schema.with_column(SmartString::from("19"), DataType::String);
        schema.with_column(SmartString::from("nhc_code"), DataType::String);
        let schema = Arc::new(schema);
        let df = CsvReader::new(cursor)
            .with_schema(Some(schema))
            .has_header(true)
            .finish()
            .expect(&format!("Unreachable: polars should've been be able to parse this. Maybe something changed at the url {}", url));
        Ok(df)
    }
}

#[derive(Error, Debug)]
pub enum StormEventBuilderError {
    #[error("network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error(
        "storm_id '{0}' does not match any known patterns for initialization \"Sandy2012\" or "
    )]
    NoMatchingPatternForStormID(String),

    #[error(
        "storm_id '{0}' does not match any known patterns for NHC code: r\"^[a-zA-Z]{{2}}\\d{{6}}$\"  "
    )]
    NoMatchingPatternForNhcCode(String),

    #[error("Polars error: {0}")]
    PolarsError(#[from] polars::prelude::PolarsError),

    #[error("No matching data found for storm: {storm_name}, year: {year}")]
    NoMatchingData { storm_name: String, year: i64 },

    #[error("Multiple matching data found for storm: {storm_name}, year: {year}")]
    MultipleMatchingData { storm_name: String, year: i64 },

    #[error("Unreachable: Unexpected multiple matching entries found for NHC code: {0}")]
    MultipleMatchingNhcCode(String),

    #[error("No matching entries found for NHC code: {0}")]
    NoMatchingNhcCode(String),

    #[error("Either storm_id or nhc_code must be set")]
    MissingArguments,

    #[error("Unitialized field on StormEventBuilder: {0}")]
    UninitializedFieldError(String),

    #[error("{0}")]
    MutuallyExclusiveArguments(String),

    // #[error("The `file_deck` field is required when requestiong realtime data.")]
    // UninitializedFileDeckError,
    #[error(transparent)]
    UrlParseError(#[from] ParseError),
    // #[error("NHCDataInventoryBuilder error: {0}")]
    // NHCDataInventoryBuilderError(#[from] NHCDataInventoryBuilderError),
}
