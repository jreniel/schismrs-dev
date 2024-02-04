use polars::prelude::*;

use super::nhc::{NHCDataInventory, NHCDataInventoryError};

pub struct StormEvent {
    track: DataFrame,
}

impl TryFrom<String> for StormEvent {
    type Error = NHCDataInventoryError;

    fn try_from(storm_id: String) -> Result<Self, Self::Error> {
        let track = NHCDataInventory::try_from(storm_id)?.dataframe()?;
        Ok(Self { track })
    }
}

impl TryFrom<(String, i32)> for StormEvent {
    type Error = NHCDataInventoryError;
    fn try_from(arg: (String, i32)) -> Result<Self, Self::Error> {
        let track = NHCDataInventory::try_from(arg)?.dataframe()?;
        Ok(Self { track })
    }
}
#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_try_from_storm_id_sandy2012() {
        StormEvent::try_from("Sandy2012".to_owned()).unwrap();
    }
}
