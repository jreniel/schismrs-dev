use crate::atcf::ATCFFileDeck;
use datetime::Year;
use polars::frame::DataFrame;
use polars::prelude::*;
use polars_lazy::prelude::*;
use regex::Regex;
use smartstring::alias::String as SmartString;
use std::io::Cursor;
use std::sync::Arc;
use thiserror::Error;
use url::Url;

pub struct StormEvent {
    name: String,
    year: Year,
    track: DataFrame,
}

#[derive(Default)]
pub struct StormEventBuilder<'a> {
    file_deck: Option<&'a ATCFFileDeck>,

    storm_id: Option<&'a String>,
}

impl<'a> StormEventBuilder<'a> {
    pub fn build(&self) -> Result<StormEvent, StormEventBuilderError> {
        // let file_deck = self.file_deck.ok_or_else(|| {
        //     StormEventBuilderError::UninitializedFieldError("file_deck".to_string())
        // })?;

        let storm_id = self.storm_id.ok_or_else(|| {
            StormEventBuilderError::UninitializedFieldError("storm_id".to_string())
        })?;

        let (name, year, track) = self.get_storm_details_from_storm_id(storm_id)?;

        Ok(StormEvent { name, year, track })
    }

    fn get_storm_details_from_storm_id(
        &self,
        storm_id: &String,
    ) -> Result<(String, Year, DataFrame), StormEventBuilderError> {
        let inventory = Self::get_nhc_storm_inventory()?;
        let (name, year) = self.get_storm_data_from_storm_id(&inventory, storm_id)?;
        unimplemented!();

        // Ok((name, year, track))
    }

    fn get_storm_data_from_storm_id(
        &self,
        inventory: &DataFrame,
        storm_id: &String,
    ) -> Result<(String, Year), StormEventBuilderError> {
        if Regex::new(r"^[a-zA-Z]{2}\d{6}$")
            .unwrap()
            .is_match(&storm_id)
        {
            let (name, year) = Self::get_storm_name_and_year_from_nhc_code(inventory, storm_id)?;
            unimplemented!();
            // Self::verify_nhc_code_exists(inventory, storm_id)?;
            // let (name, year) = Self::get_name_and_year_from_nhc_code(storm_id)?;
            // Ok(storm_id.to_string())
        } else if Regex::new(r"^[a-zA-Z]*\d{4}$").unwrap().is_match(&storm_id) {
            let name = storm_id[..storm_id.len() - 4].to_string();
            let year = Year(storm_id[storm_id.len() - 4..].parse::<i64>().map_err(|_| {
                StormEventBuilderError::NoMatchingPatternForStormID(storm_id.clone())
            })?);
            let nhc_code = Self::get_nhc_code_from_storm_name_and_year(inventory, &name, &year);
            unimplemented!();
            // let track = self.get_track_from_nhc_code(&nhc_code)?;
            // Ok((name, year, track))
        } else {
            Err(StormEventBuilderError::NoMatchingPatternForStormID(
                storm_id.to_string(),
            ))
        }
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
                    .eq(lit(storm_name.to_uppercase()))
                    .and(col("year").eq(lit(year.0))),
            )
            .collect()?;
        if some_coll.height() != 1 {
            return Err(StormEventBuilderError::MultipleMatchingData {
                storm_name: storm_name.to_owned(),
                year: year.0,
            });
        } else if some_coll.height() == 0 {
            return Err(StormEventBuilderError::NoMatchingData {
                storm_name: storm_name.to_owned(),
                year: year.0,
            });
        }
        let nhc_code_column = some_coll.column("nhc_code")?;

        let nhc_code_value = nhc_code_column.get(0);

        let nhc_code = nhc_code_value?.to_string();

        Ok(nhc_code)
    }
    fn get_storm_name_and_year_from_nhc_code(
        inventory: &DataFrame,
        nhc_code: &str,
    ) -> Result<(String, Year), StormEventBuilderError> {
        let some_coll = inventory
            .clone()
            .lazy()
            .filter(col("nhc_code").eq(lit(nhc_code.to_uppercase())))
            .collect()?;
        if some_coll.height() > 1 {
            return Err(StormEventBuilderError::MultipleMatchingNhcCode(
                nhc_code.to_owned(),
            ));
        } else if some_coll.height() == 0 {
            return Err(StormEventBuilderError::NoMatchingNhcCode(
                nhc_code.to_owned(),
            ));
        }
        dbg!(some_coll);
        unimplemented!();
        // Ok(())
    }
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
    // fn get_atcf_url(&self, storm_name: &str, year: &Year) -> Result<Url, NHCDataInventoryError> {
    //     // let nhc_dir =
    //     // match mode {
    //     //     AtcfMode::Historical => {
    //     //         if year.is_none() {
    //     //             panic!("NHC storm code not given");
    //     //         }
    //     //         (format!("archive/{}", year.unwrap()), ".dat.gz".to_string())
    //     //     },
    //     //     AtcfMode::Realtime => {
    //     //         match file_deck {
    //     //             AtcfFileDeck::Advisory => ("aid_public".to_string(), ".dat.gz".to_string()),
    //     //             AtcfFileDeck::Best => ("btk".to_string(), ".dat".to_string()),
    //     //             AtcfFileDeck::Fixed => ("fix".to_string(), ".dat".to_string()),
    //     //             // ... handle other variants
    //     //         }
    //     //     },
    //     // }
    //     if let year
    //     let mut url = format!("ftp://ftp.nhc.noaa.gov/atcf/{}/", nhc_dir);
    //     if let Some(code) = nhc_code {
    //         url.push_str(&format!(
    //             "{}{}{}",
    //             match self.file_deck {
    //                 Some(&ATCFFileDeck::ADVISORY) => "advisory",
    //                 Some(&ATCFFileDeck::BEST) => "best",
    //                 Some(&ATCFFileDeck::FIXED) => "fixed",
    //                 // ... map other variants
    //             },
    //             code.to_lowercase(),
    //             suffix
    //         ));
    //     }

    //     url
    // }
}

#[derive(Error, Debug)]
pub enum StormEventBuilderError {
    #[error("network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error(
        "storm_id '{0}' does not match any known patterns for initialization \"Sandy2012\" or "
    )]
    NoMatchingPatternForStormID(String),

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

    // #[error("NHCDataInventoryBuilder error: {0}")]
    // NHCDataInventoryBuilderError(#[from] NHCDataInventoryBuilderError),
    #[error("Either storm_id or nhc_code must be set")]
    MissingArguments,

    #[error("Unitialized field on StormEventBuilder: {0}")]
    UninitializedFieldError(String),

    #[error("{0}")]
    MutuallyExclusiveArguments(String),
}
