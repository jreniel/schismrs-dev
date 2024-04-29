
use linked_hash_set::LinkedHashSet;
use std::iter::zip;

static MAJOR_CONSTITUENTS: &[&'static str] = &["Q1", "O1", "P1", "K1", "N2", "M2", "S2", "K2"];
static MINOR_CONSTITUENTS: &[&'static str] = &["Mm", "Mf", "M4", "MN4", "MS4", "2N2", "S1"];

macro_rules! define_constituents_config {
    ( $( $name:ident ),* ) => {
        #[allow(non_snake_case)]
        #[derive(Default, Debug)]
        pub struct ConstituentsConfig {
            $( pub $name: bool, )*
        }

        impl ConstituentsConfig {
            pub fn field_names() -> Vec<&'static str> {
                vec![$( stringify!($name), )*]
            }

            pub fn values(&self) -> Vec<bool> {
                vec![$( self.$name, )*]
            }

            pub fn set_by_name(&mut self, field_name: &str, value: bool) {
                // Check if the field name starts with a digit and prepend an underscore if it does
                let adjusted_field_name = if field_name.chars().next().map_or(false, |c| c.is_digit(10)) {
                    format!("_{}", field_name)
                } else {
                    String::from(field_name)
                };

                // Use the adjusted field name in the match
                match adjusted_field_name.as_str() {
                    // Using stringify! in a match requires the names to be known at compile time,
                    // Assuming $name is a macro variable, replace this with your actual fields
                    $( stringify!($name) => self.$name = value, )*
                    _ => panic!("Field name does not exist in ConstituentsConfig"),
                }
            }
            pub fn all() -> Self {
                let mut this = Self::default();
                for cnst in MAJOR_CONSTITUENTS.iter() {
                    this.set_by_name(cnst, true)
                }
                for cnst in MINOR_CONSTITUENTS.iter() {
                    this.set_by_name(cnst, true)
                }
                this
            }
            pub fn major() -> Self {
                let mut this = Self::default();
                for cnst in MAJOR_CONSTITUENTS.iter() {
                    this.set_by_name(cnst, true)
                }
                this
            }
            pub fn minor() -> Self {
                let mut this = Self::default();
                for cnst in MINOR_CONSTITUENTS.iter() {
                    this.set_by_name(cnst, true)
                }
                this
            }
            pub fn get_active_potential_constituents(&self) -> LinkedHashSet<String> {
                let mut apc = LinkedHashSet::new();
                for (field_name, field_value) in zip(Self::field_names(), self.values()) {
                    if field_value == true {
                        // The problem is that the static tables in tidefac.rs do not include
                        // tidal_species_type, tidal_potential_amplitudes and/or orbital
                        // frequencies.
                        if MAJOR_CONSTITUENTS.contains(&field_name) {
                            apc.insert(field_name.to_string());
                        }
                    }
                }
                apc
            }
            pub fn get_active_forcing_constituents(&self) -> LinkedHashSet<String> {
                let mut afc = LinkedHashSet::new();
                for (field_name, field_value) in zip(Self::field_names(), self.values()) {
                    if field_value == true {
                        afc.insert(field_name.to_string());
                    }
                }
                afc
            }
        }
    }
}

// Using the macro to define the struct
define_constituents_config! {
    Q1, O1, P1, K1, N2, M2, S2, K2, Mm, Mf, M4, MN4, MS4, _2N2, S1
}

#[derive(Debug)]
pub enum TidalDatabase {
    TPXO,
    HAMTIDE,
    FES,
}

#[derive(Debug)]
pub enum TimeSeriesDatabase {
    HYCOM,
}

#[derive(Debug)]
pub struct TidesConfig {
    pub constituents: ConstituentsConfig,
    pub database: TidalDatabase,
}

#[derive(Debug)]
pub struct SpaceVaryingTimeSeriesConfig {
    // pub data: BTreeMap<u32, BTreeMap<DateTime<Utc>, f64>>,
    pub database: TimeSeriesDatabase,
}

// impl SpaceVaryingTimeSeriesConfig {
//     fn from_database(database: &TimeSeriesDatabase) -> Self {
//         Self { data, database }
//     }
// }
