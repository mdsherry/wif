use std::{
    collections::{BTreeMap, BTreeSet},
    num::{ParseFloatError, ParseIntError},
    str::FromStr,
};

pub mod wifparse;

mod wif;
pub use wif::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Treadle(pub u32);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Shaft(u32);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Warp(u32);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Weft(u32);
impl From<u32> for Treadle {
    fn from(value: u32) -> Self {
        Treadle(value)
    }
}
impl From<u32> for Shaft {
    fn from(value: u32) -> Self {
        Shaft(value)
    }
}
impl From<u32> for Warp {
    fn from(value: u32) -> Self {
        Warp(value)
    }
}
impl From<u32> for Weft {
    fn from(value: u32) -> Self {
        Weft(value)
    }
}
impl FromStr for Treadle {
    type Err = ParseIntError;

    fn from_str(s: &str) -> std::prelude::v1::Result<Self, Self::Err> {
        s.parse::<u32>().map(Treadle)
    }
}
impl FromStr for Shaft {
    type Err = ParseIntError;

    fn from_str(s: &str) -> std::prelude::v1::Result<Self, Self::Err> {
        s.parse::<u32>().map(Shaft)
    }
}
impl FromStr for Warp {
    type Err = ParseIntError;

    fn from_str(s: &str) -> std::prelude::v1::Result<Self, Self::Err> {
        s.parse::<u32>().map(Warp)
    }
}
impl FromStr for Weft {
    type Err = ParseIntError;

    fn from_str(s: &str) -> std::prelude::v1::Result<Self, Self::Err> {
        s.parse::<u32>().map(Weft)
    }
}

impl std::fmt::Display for Treadle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
impl std::fmt::Display for Warp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
impl std::fmt::Display for Weft {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

type Table<S, T> = BTreeMap<S, T>;
pub type Result<T, E = WifError> = std::result::Result<T, E>;

fn liftplan_from_threading_and_treadle(
    treadling: Option<&BTreeMap<Weft, BTreeSet<Treadle>>>,
    tieup: Option<&BTreeMap<Treadle, BTreeSet<Shaft>>>,
) -> Option<BTreeMap<Weft, BTreeSet<Shaft>>> {
    let mut lift_plan: BTreeMap<_, _> = Default::default();
    for (&weft_row, treadles) in treadling? {
        let mut rv: BTreeSet<Shaft> = BTreeSet::new();
        for treadle in treadles {
            if let Some(tie) = tieup?.get(treadle) {
                rv.extend(tie);
            }
        }
        lift_plan.insert(weft_row, rv);
    }
    Some(lift_plan)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WarpOrWeft {
    Warp,
    Weft,
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum WifError {
    #[error("Section [{section}] is missing required field '{field}'")]
    MissingRequiredField { section: String, field: String },
    #[error("Error parsing [{section}].{field}: {err}")]
    FieldParseError {
        section: String,
        field: String,
        err: Box<WifError>,
    },

    #[error(transparent)]
    InvalidDate {
        #[from]
        error: chrono::ParseError,
    },
    #[error(transparent)]
    InvalidNumber {
        #[from]
        error: ParseIntError,
    },
    #[error(transparent)]
    InvalidFloat {
        #[from]
        error: ParseFloatError,
    },
    #[error("Expected pair, but saw {saw}")]
    ExpectedPair { saw: String },
    #[error("Expected boolean, but saw {saw}")]
    ExpectedBool { saw: String },
    #[error("Section {section} was indicated in CONTENTS, but could not be found")]
    MissingSection { section: String },
    #[error("Could not parse table key for section [{section}]: saw {key}")]
    CouldNotParseTableKey { section: String, key: String },
    #[error("Lift plan does not match treadling and tieup")]
    LiftPlanDoesNotMatchTreadling,
    #[error("Colors must be three numbers")]
    ColorsMustBeThreeParts,
}

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub red: u32,
    pub green: u32,
    pub blue: u32,
}

trait WifContext {
    fn add_context(self, section: &str, field: &str) -> Self;
}
impl<T> WifContext for Result<T> {
    fn add_context(self, section: &str, field: &str) -> Self {
        self.map_err(|e| WifError::FieldParseError {
            section: section.into(),
            field: field.into(),
            err: Box::new(e),
        })
    }
}
