use ndarray::Array;
use ndarray::Array1;
use ndarray_stats::QuantileExt;
use schismrs_hgrid::Hgrid;
use std::fmt;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use thiserror::Error;

pub struct SZ {
    sigma: Array1<f64>,
    z_array: Array1<f64>,
    theta_f: f64,
    theta_b: f64,
    hc: f64, // also known as critical layer depth
}

impl SZ {
    pub fn write_to_file(&self, filename: &PathBuf) -> std::io::Result<()> {
        let mut file = File::create(filename)?;
        write!(file, "{}", self)?;
        Ok(())
    }
    pub fn ivcor(&self) -> usize {
        2
    }
    pub fn nvrt(&self) -> usize {
        self.sigma.len() + self.z_array.len() - 1
    }
}

impl fmt::Display for SZ {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}\n", self.ivcor())?;
        write!(f, "{} ", self.nvrt())?;
        let kz = self.z_array.len();
        write!(f, "{} ", &kz)?;
        write!(f, "{}\n", -self.z_array[self.z_array.len() - 1])?;
        write!(f, "Z levels\n")?;
        for (i, val) in self.z_array.iter().enumerate() {
            write!(f, "{} {}\n", i + 1, val)?;
        }
        write!(f, "S levels\n")?;
        write!(f, "{} {} {}\n", self.hc, self.theta_b, self.theta_f)?;
        for (i, val) in self.sigma.iter().enumerate() {
            write!(f, "{} {}\n", i + &kz, val)?;
        }
        Ok(())
    }
}

#[derive(Default)]
pub struct SZBuilder<'a> {
    hgrid: Option<&'a Hgrid>,
    slevels: Option<&'a usize>,
    zlevels: Option<&'a Vec<f64>>,
    theta_b: Option<&'a f64>,
    theta_f: Option<&'a f64>,
    critical_depth: Option<&'a f64>,
}

impl<'a> SZBuilder<'a> {
    pub fn build(&self) -> Result<SZ, SZBuilderError> {
        let hgrid = self
            .hgrid
            .ok_or_else(|| SZBuilderError::UninitializedFieldError("hgrid".to_string()))?;
        let slevels = self
            .slevels
            .as_ref()
            .ok_or_else(|| SZBuilderError::UninitializedFieldError("slevels".to_string()))?;
        let theta_f = self
            .theta_f
            .as_ref()
            .ok_or_else(|| SZBuilderError::UninitializedFieldError("theta_f".to_string()))?;
        Self::validate_theta_f(theta_f)?;
        let theta_b = self
            .theta_b
            .as_ref()
            .ok_or_else(|| SZBuilderError::UninitializedFieldError("theta_b".to_string()))?;
        Self::validate_theta_b(theta_b)?;
        let critical_depth = self
            .critical_depth
            .as_ref()
            .ok_or_else(|| SZBuilderError::UninitializedFieldError("critical_depth".to_string()))?;
        Self::validate_critical_depth(critical_depth)?;
        let depths = hgrid.depths();
        let deepest_point = depths.min()?;
        let z_array: Array1<f64> = match &self.zlevels {
            None => Array1::from_vec(vec![*deepest_point]),
            Some(zlevels) => {
                Self::validate_z_levels(deepest_point, zlevels)?;
                Array1::from_vec(zlevels.to_vec())
            }
        };
        let sigma = Array::linspace(-1., 0., **slevels);
        Ok(SZ {
            sigma,
            z_array,
            theta_f: **theta_f,
            theta_b: **theta_b,
            hc: **critical_depth,
        })
    }
    fn validate_theta_b(theta_b: &f64) -> Result<(), SZBuilderError> {
        if !(0.0 <= *theta_b && *theta_b <= 1.0) {
            return Err(SZBuilderError::InvalidThetaB(*theta_b));
        };
        Ok(())
    }
    fn validate_critical_depth(critical_depth: &f64) -> Result<(), SZBuilderError> {
        if *critical_depth < 5. {
            return Err(SZBuilderError::InvalidCriticalDepth(*critical_depth));
        };
        Ok(())
    }
    fn validate_theta_f(theta_f: &f64) -> Result<(), SZBuilderError> {
        if !(*theta_f >= 0.) {
            return Err(SZBuilderError::InvalidThetaF(*theta_f));
        };
        Ok(())
    }
    fn validate_z_levels(deepest_point: &f64, zlevels: &Vec<f64>) -> Result<(), SZBuilderError> {
        if !zlevels.iter().all(|&val| val <= 0.0) {
            return Err(SZBuilderError::InvalidZLevels);
        }
        if zlevels.len() > 1 {
            if !zlevels.windows(2).all(|pair| pair[0] < pair[1]) {
                return Err(SZBuilderError::InvalidZLevels);
            }
        }
        if zlevels[0] > *deepest_point {
            return Err(SZBuilderError::InvalidZLevelsValues(
                *deepest_point,
                zlevels[0],
            ));
        }
        Ok(())
    }
    pub fn hgrid(&mut self, hgrid: &'a Hgrid) -> &mut Self {
        self.hgrid = Some(hgrid);
        self
    }
    pub fn slevels(&mut self, slevels: &'a usize) -> &mut Self {
        self.slevels = Some(slevels);
        self
    }
    pub fn zlevels(&mut self, zlevels: &'a Vec<f64>) -> &mut Self {
        self.zlevels = Some(zlevels);
        self
    }
    pub fn theta_b(&mut self, theta_b: &'a f64) -> &mut Self {
        self.theta_b = Some(theta_b);
        self
    }
    pub fn theta_f(&mut self, theta_f: &'a f64) -> &mut Self {
        self.theta_f = Some(theta_f);
        self
    }
    pub fn critical_depth(&mut self, critical_depth: &'a f64) -> &mut Self {
        self.critical_depth = Some(critical_depth);
        self
    }
}

#[derive(Error, Debug)]
pub enum SZBuilderError {
    #[error("Unitialized field on SZBuilder: {0}")]
    UninitializedFieldError(String),
    #[error(transparent)]
    MinMaxError(#[from] ndarray_stats::errors::MinMaxError),
    #[error("zlevels must be all negative and increasing")]
    InvalidZLevels,
    #[error("The first point of zlevels must be smaller or equal to the deepest point in the mesh ({0}) but got {1}")]
    InvalidZLevelsValues(f64, f64),
    #[error("theta_b must be in [0., 1.], but got {0}")]
    InvalidThetaB(f64),
    #[error("theta_f must be larger or equal than 0, but got {0}")]
    InvalidThetaF(f64),
    #[error("critical depth must be larger or equal than 5., but got {0}")]
    InvalidCriticalDepth(f64),
}
