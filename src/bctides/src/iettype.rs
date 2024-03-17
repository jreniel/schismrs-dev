use chrono::{DateTime, Utc};
use std::collections::BTreeMap;

#[allow(non_snake_case)]
#[derive(Default)]
pub struct ConstituentsConfig {
    pub Q1: bool,
    pub O1: bool,
    pub P1: bool,
    pub K1: bool,
    pub N2: bool,
    pub M2: bool,
    pub S2: bool,
    pub K2: bool,
    pub Mm: bool,
    pub Mf: bool,
    pub M4: bool,
    pub MN4: bool,
    pub MS4: bool,
    pub _2N2: bool,
}

impl ConstituentsConfig {
    fn all() -> Self {
        ConstituentsConfig {
            Q1: true,
            O1: true,
            P1: true,
            K1: true,
            N2: true,
            M2: true,
            S2: true,
            K2: true,
            Mm: true,
            Mf: true,
            M4: true,
            MN4: true,
            MS4: true,
            _2N2: true,
        }
    }
    fn major() -> Self {
        ConstituentsConfig {
            Q1: true,
            O1: true,
            P1: true,
            K1: true,
            N2: true,
            M2: true,
            S2: true,
            K2: true,
            Mm: false,
            Mf: false,
            M4: false,
            MN4: false,
            MS4: false,
            _2N2: false,
        }
    }
    fn minor() -> Self {
        ConstituentsConfig {
            Q1: false,
            O1: false,
            P1: false,
            K1: false,
            N2: false,
            M2: false,
            S2: false,
            K2: false,
            Mm: true,
            Mf: true,
            M4: true,
            MN4: true,
            MS4: true,
            _2N2: true,
        }
    }
}

pub enum TidalDatabase {
    TPXO,
    HAMTIDE,
    FES,
}

pub enum TimeSeriesDatabase {
    HYCOM,
}

pub struct TidesConfig {
    constituents: ConstituentsConfig,
    database: TidalDatabase,
}

pub struct SpaceVaryingTimeSeriesConfig {
    data: BTreeMap<u32, BTreeMap<DateTime<Utc>, f64>>,
    database: TimeSeriesDatabase,
}

pub enum ElevationConfig {
    UniformTimeSeries(BTreeMap<DateTime<Utc>, f64>),
    ConstantValue(f64),
    Tides(TidesConfig),
    SpaceVaryingTimeSeries(SpaceVaryingTimeSeriesConfig),
    TidesAndSpaceVaryingTimeSeries {
        tides: TidesConfig,
        time_series: SpaceVaryingTimeSeriesConfig,
    },
}
