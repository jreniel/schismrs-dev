use crate::ElevationConfig;
use chrono::{DateTime, Utc};
use schismrs_hgrid::Hgrid;
use std::collections::BTreeMap;
use std::fmt;
use thiserror::Error;

#[derive(Debug)]
pub struct Bctides<'a> {
    // hgrid: &'a Hgrid,
    start_date: &'a DateTime<Utc>,
    tidal_potential_cutoff_depth: &'a f64,
}

impl<'a> Bctides<'a> {
    fn ntip(&self) -> u8 {
        todo!();
    }
    fn tip_dp(&self) -> &f64 {
        self.tidal_potential_cutoff_depth
    }
}

impl fmt::Display for Bctides<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}\n", self.start_date)?;
        write!(f, "{} {}\n", self.ntip(), self.tip_dp())?;
        Ok(())
    }
}

#[derive(Default)]
pub struct BctidesBuilder<'a> {
    start_date: Option<&'a DateTime<Utc>>,
    tidal_potential_cutoff_depth: Option<&'a f64>,
    boundary_forcing_config: Option<&'a BoundaryForcingConfig>,
}

impl<'a> BctidesBuilder<'a> {
    pub fn build(&self) -> Result<Bctides, BctidesBuilderError> {
        let start_date = self.start_date.ok_or_else(|| {
            BctidesBuilderError::UninitializedFieldError("start_date".to_string())
        })?;
        let tidal_potential_cutoff_depth = self.tidal_potential_cutoff_depth.ok_or_else(|| {
            BctidesBuilderError::UninitializedFieldError("tidal_potential_cutoff_depth".to_string())
        })?;
        Self::validate(tidal_potential_cutoff_depth)?;
        Ok(Bctides {
            // hgrid,
            start_date,
            tidal_potential_cutoff_depth,
        })
    }
    pub fn start_date(&mut self, start_date: &'a DateTime<Utc>) -> &mut Self {
        self.start_date = Some(start_date);
        self
    }
    pub fn tidal_potential_cutoff_depth(
        &mut self,
        tidal_potential_cutoff_depth: &'a f64,
    ) -> &mut Self {
        self.tidal_potential_cutoff_depth = Some(tidal_potential_cutoff_depth);
        self
    }
    fn validate(tidal_potential_cutoff_depth: &'a f64) -> Result<(), BctidesBuilderError> {
        Self::validate_tidal_potential_cutoff_depth(tidal_potential_cutoff_depth)?;
        Ok(())
    }
    fn validate_tidal_potential_cutoff_depth(
        tidal_potential_cutoff_depth: &f64,
    ) -> Result<(), BctidesBuilderError> {
        if tidal_potential_cutoff_depth < &0. {
            return Err(BctidesBuilderError::InvalidTidalPotentialCutoffDepth);
        }
        Ok(())
    }
}
#[derive(Error, Debug)]
pub enum BctidesBuilderError {
    #[error("Unitialized field on BctidesBuilder: {0}")]
    UninitializedFieldError(String),
    #[error("tidal_potential_cutoff_depth must be >= 0.")]
    InvalidTidalPotentialCutoffDepth,
}

pub struct BoundaryForcingConfig<'a> {
    hgrid: &'a Hgrid,
    elevation: Option<&'a BTreeMap<u32, ElevationConfig>>,
}

pub struct BoundaryForcingConfigBuilder<'a> {
    hgrid: Option<&'a Hgrid>,
    elevation: Option<&'a BTreeMap<u32, ElevationConfig>>,
}

impl<'a> BoundaryForcingConfigBuilder<'a> {
    pub fn build(&self) -> Result<BoundaryForcingConfig, BoundaryForcingConfigBuilderError> {
        let hgrid = self.hgrid.ok_or_else(|| {
            BoundaryForcingConfigBuilderError::UninitializedFieldError("hgrid".to_string())
        })?;
        Self::validate(hgrid, self.elevation)?;
        Ok(BoundaryForcingConfig {
            hgrid,
            elevation: self.elevation,
        })
    }
    pub fn hgrid(&mut self, hgrid: &'a Hgrid) -> &mut Self {
        self.hgrid = Some(hgrid);
        self
    }
    pub fn elevation(&mut self, elevation: &'a BTreeMap<u32, ElevationConfig>) -> &mut Self {
        self.elevation = Some(elevation);
        self
    }
}
#[derive(Error, Debug)]
pub enum BoundaryForcingConfigBuilderError {
    #[error("Unitialized field on BoundaryForcingConfigBuilder: {0}")]
    UninitializedFieldError(String),
    // #[error("tidal_potential_cutoff_depth must be >= 0.")]
    // InvalidTidalPotentialCutoffDepth,
}
