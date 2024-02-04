// use chrono::{DateTime, Duration, Utc};
use super::atcf::ATCFFileDeck;
use derive_builder::Builder;
use polars::prelude::*;
use polars_lazy::prelude::*;
use regex::Regex;
use reqwest;
use smartstring::alias::String as SmartString;
use std::io::Cursor;
use std::sync::Arc;
use thiserror::Error;

#[derive(Builder)]
pub struct NHCDataInventory {
    #[builder(setter(skip))]
    inventory: DataFrame,
    #[builder(default = "ATCFFileDeck::ADVISORY")] // redundant to internal_build
    file_deck: ATCFFileDeck,
    #[builder(setter(into))]
    nhc_code: String,
    #[builder(setter(strip_option), private)]
    storm_id: Option<String>,
}

impl NHCDataInventoryBuilder {
    fn internal_build(&self) -> Result<NHCDataInventory, NHCDataInventoryError> {
        let inventory = NHCDataInventory::get_nhc_storm_inventory()?;
        let file_deck = self
            .file_deck
            .clone()
            .unwrap_or_else(|| ATCFFileDeck::ADVISORY);
        // Check if both storm_id and nhc_code are set
        if self.storm_id.is_some() && self.nhc_code.is_some() {
            return Err(NHCDataInventoryError::MutuallyExclusiveArguments(
                "storm_id and nhc_code cannot both be set.".to_string(),
            ));
        }

        let nhc_code = if let Some(storm_id) = self.storm_id {
            NHCDataInventory::get_nhc_code_from_storm_id(&inventory, storm_id.unwrap())?
        } else {
            self.nhc_code
                .clone()
                .ok_or(NHCDataInventoryBuilderError::UninitializedField("nhc_code"))?
        };
        NHCDataInventory::verify_nhc_code_exists(&inventory, &nhc_code)?;
        let this_inventory = NHCDataInventory {
            inventory,
            file_deck,
            nhc_code,
            storm_id: None,
        };
        Ok(this_inventory)
    }
}

#[derive(Error, Debug)]
pub enum NHCDataInventoryError {
    #[error("network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error(
        "storm_id '{0}' does not match any known patterns for initialization \"Sandy2012\" or "
    )]
    NoMatchingPatternForStormID(String),

    #[error("Polars error: {0}")]
    PolarsError(#[from] polars::prelude::PolarsError),

    #[error("No matching data found for storm: {storm_name}, year: {year}")]
    NoMatchingData { storm_name: String, year: i32 },

    #[error("Multiple matching data found for storm: {storm_name}, year: {year}")]
    MultipleMatchingData { storm_name: String, year: i32 },

    #[error("Unreachable: Unexpected multiple matching entries found for NHC code: {0}")]
    MultipleMatchingNhcCode(String),

    #[error("No matching entries found for NHC code: {0}")]
    NoMatchingNhcCode(String),

    #[error("NHCDataInventoryBuilder error: {0}")]
    NHCDataInventoryBuilderError(#[from] NHCDataInventoryBuilderError),

    #[error("{0}")]
    MutuallyExclusiveArguments(String),
}

impl NHCDataInventory {
    pub fn new(nhc_code: String) -> Result<Self, NHCDataInventoryError> {
        Ok(NHCDataInventoryBuilder::default()
            .nhc_code(nhc_code)
            .build()?)
    }

    fn get_nhc_storm_inventory() -> Result<DataFrame, NHCDataInventoryError> {
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

    fn get_nhc_code_from_storm_name_and_year(
        inventory: &DataFrame,
        storm_name: &str,
        year: &i32,
    ) -> Result<String, NHCDataInventoryError> {
        let some_coll = inventory
            .clone()
            .lazy()
            .filter(
                col("name")
                    .eq(lit(storm_name.to_uppercase()))
                    .and(col("year").eq(lit(*year))),
            )
            .collect()?;
        if some_coll.height() != 1 {
            return Err(NHCDataInventoryError::MultipleMatchingData {
                storm_name: storm_name.to_owned(),
                year: *year,
            });
        } else if some_coll.height() == 0 {
            return Err(NHCDataInventoryError::NoMatchingData {
                storm_name: storm_name.to_owned(),
                year: *year,
            });
        }
        // Assuming you know that nhc_code is in a column named "nhc_code"
        let nhc_code_column = some_coll.column("nhc_code")?;

        // Get the first (and only) value from the column
        let nhc_code_value = nhc_code_column.get(0);

        // Convert it to a String (adjust this based on the actual data type)
        let nhc_code = nhc_code_value?.to_string();
        // let nhc_code: String;
        Ok(nhc_code)
    }

    fn verify_nhc_code_exists(
        inventory: &DataFrame,
        nhc_code: &str,
    ) -> Result<(), NHCDataInventoryError> {
        let some_coll = inventory
            .clone()
            .lazy()
            .filter(col("nhc_code").eq(lit(nhc_code.to_uppercase())))
            .collect()?;
        if some_coll.height() > 1 {
            return Err(NHCDataInventoryError::MultipleMatchingNhcCode(
                nhc_code.to_owned(),
            ));
        } else if some_coll.height() == 0 {
            return Err(NHCDataInventoryError::NoMatchingNhcCode(
                nhc_code.to_owned(),
            ));
        }
        Ok(())
    }

    pub fn from_storm_id(storm_id: String) -> Result<Self, NHCDataInventoryError> {
        unimplemented!()
    }

    fn get_nhc_code_from_storm_id(
        inventory: &DataFrame,
        storm_id: String,
    ) -> Result<String, NHCDataInventoryError> {
        if Regex::new(r"^[a-zA-Z]{2}\d{6}$")
            .unwrap()
            .is_match(&storm_id)
        {
            // in this case, the storm_id passes the regex check for an NHC code,
            // so just check if it's  a valid one against the inventory.
            Self::verify_nhc_code_exists(inventory, &storm_id)?;
            Ok(storm_id)
        } else if Regex::new(r"^[a-zA-Z]*\d{4}$").unwrap().is_match(&storm_id) {
            let name = storm_id[..storm_id.len() - 4].to_string();
            let year = storm_id[storm_id.len() - 4..].parse::<i32>().map_err(|_| {
                NHCDataInventoryError::NoMatchingPatternForStormID(storm_id.clone())
            })?;
            Ok(Self::get_nhc_code_from_storm_name_and_year(
                inventory, &name, &year,
            )?)
        } else {
            Err(NHCDataInventoryError::NoMatchingPatternForStormID(storm_id))
        }
    }

    //fn get_nhc_dir(&self) -> String {
    //    //first determine if the year
    //}

    //pub fn get_atcf_url(&self) -> Url {
    //    let nhc_dir = self.get_nhc_dir();
    //    let mut url = format!("ftp://ftp.nhc.noaa.gov/atcf/{}/", nhc_dir);
    //    url
    //}

    pub fn dataframe(&self) -> Result<DataFrame, NHCDataInventoryError> {
        // let url = self.get_atcf_url();
        let df: DataFrame;
        Ok(df)
    }

    // pub fn get_atcf_dataframe_from_storm_name_and_year(&self, storm_name: &str, year: &i32) -> Result<DataFrame, NHCDataInventoryError> {
    //     let nhc_code = self.get_nhc_code_from_storm_name_and_year(storm_name, year)?;
    //     self.get_atcf_dataframe()
    // }

    // pub fn track_from_storm_name_and_year(&self, storm_name: &str, year: &i32) -> Result<DataFrame, NHCDataInventoryError> {
    //     Ok(self.get_atcf_dataframe_from_storm_name_and_year(storm_name, year)?)
    // }

    // pub fn get_atcf_url(&self, storm_name: &str, year: &i32) -> Result<Url, NHCDataInventoryError> {
    //     let nhc_dir =
    //     match mode {
    //         AtcfMode::Historical => {
    //             if year.is_none() {
    //                 panic!("NHC storm code not given");
    //             }
    //             (format!("archive/{}", year.unwrap()), ".dat.gz".to_string())
    //         },
    //         AtcfMode::Realtime => {
    //             match file_deck {
    //                 AtcfFileDeck::Advisory => ("aid_public".to_string(), ".dat.gz".to_string()),
    //                 AtcfFileDeck::Best => ("btk".to_string(), ".dat".to_string()),
    //                 AtcfFileDeck::Fixed => ("fix".to_string(), ".dat".to_string()),
    //                 // ... handle other variants
    //             }
    //         },
    //     }
    //     let mut url = format!("ftp://ftp.nhc.noaa.gov/atcf/{}/", nhc_dir);
    //     if let Some(code) = nhc_code {
    //         url.push_str(&format!(
    //             "{}{}{}",
    //             match file_deck {
    //                 AtcfFileDeck::Advisory => "advisory",
    //                 AtcfFileDeck::Best => "best",
    //                 AtcfFileDeck::Fixed => "fixed",
    //                 // ... map other variants
    //             },
    //             code.to_lowercase(),
    //             suffix
    //         ));
    //     }

    //     url
    // }

    // pub fn get_atcf_dataframe(
    //     &self,
    //     storm_name: &str,
    //     year: &i32,
    // ) -> Result<DataFrame, NHCDataInventoryError> {
    //     let url = self.get_atcf_url(storm_name, year)?;
    //     Ok(df)
    // }
}

impl TryFrom<String> for NHCDataInventory {
    type Error = NHCDataInventoryError;
    fn try_from(storm_id: String) -> Result<Self, Self::Error> {
        let nhc_data = Self::new(storm_id)?;
        Ok(nhc_data)
    }
}

impl TryFrom<(String, i32)> for NHCDataInventory {
    type Error = NHCDataInventoryError;
    fn try_from(arg: (String, i32)) -> Result<Self, Self::Error> {
        let (name, year) = arg;
        let inventory = Self::get_nhc_storm_inventory()?;
        let storm_id = Self::get_nhc_code_from_storm_name_and_year(&inventory, &name, &year)?;
        let nhc_data = Self::new(storm_id)?;
        // let nhc_data = Self {
        //     inventory,
        //     nhc_code,
        // }?;
        // nhc_data.get_nhc_code_from_storm_name_and_year(&name, &year)?;
        Ok(nhc_data) // Return the manipulated nhc_data
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_get_nhc_storm_df_try_from_tuple() {
        let storm_name = "Sandy".to_owned();
        let storm_year = 2012;
        NHCDataInventory::try_from((storm_name, storm_year)).unwrap();
    }
    fn test_get_nhc_storm_df_builder_from_storm_id() {
        NHCDataInventoryBuilder::default().storm_id("Sandy2012".to_owned());
    }
}
