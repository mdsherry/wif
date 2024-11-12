use std::{collections::BTreeSet, str::FromStr};

use configparser::ini::Ini;

use crate::{wifparse::WifParse, Color, Shaft, Table, Treadle, WifContext, WifError};

use super::{get_field, get_required_field, sections, Section, WifHeader};

pub(crate) trait WifSection {
    const NAME: &str;
    type Output;
    fn write(value: &Self::Output, ini: &mut Ini);
    fn read(ini: &Ini) -> Result<Self::Output, crate::WifError>;
}

macro_rules! read_fields {
    (@single $ini:ident [$($body:tt)*] $(,)?) => {
        Self::Output {
            $($body)*
        }
    };
    (@single $ini:ident [$($body:tt)*] ? $name:ident : $field:literal , $($rest:tt)*) => {
        read_fields!(
            @single $ini
            [$($body)* $name: get_field($ini, Self::NAME, $field)? ,]
            $($rest)*)
    };
    (@single $ini:ident [$($body:tt)*] $name:ident : $field:literal , $($rest:tt)*) => {
        read_fields!(
            @single $ini
            [$($body)* $name: get_required_field($ini, Self::NAME, $field)? ,]
            $($rest)*)
    };
    ($ini:ident , $($blah:tt)*) => {
        read_fields!(@single $ini [] $($blah)* , )
    };
}

macro_rules! write_fields {
    (@single $s:ident $value:ident $(,)?) => {

    };
    (@single $s:ident $value:ident $(?)? $name:ident : $field:literal , $($rest:tt)*) => {
        $s.write($field, &$value.$name);
        write_fields!(
            @single $s $value
            $($rest)*)
    };
    ($ini:ident $value:ident, $($blah:tt)*) => {
        let mut s = Section::new($ini, Self::NAME);
        if Self::NAME != "WIF" {
            s.record_usage();
        }
        write_fields!(@single s $value $($blah)* , )
    };
}

macro_rules! wr_fields {
    ($($blah:tt)*) => {
        fn write(value: &Self::Output, ini: &mut Ini) {
            write_fields!(
                ini value, $($blah)*
            );
        }


        fn read(ini: &Ini) -> Result<Self::Output, crate::WifError> {
            Ok(read_fields! {
                ini,
                $($blah)*
            })
        }
    };
}

macro_rules! wr_table {
    () => {
        fn write(value: &Self::Output, ini: &mut Ini) {
            let mut s = Section::new(ini, Self::NAME);
            if Self::NAME != "WIF" {
                s.record_usage();
            }
            s.write_table(value);
        }

        fn read(ini: &Ini) -> Result<Self::Output, crate::WifError> {
            parse_table(ini, Self::NAME)
        }
    };
}

pub(crate) struct Wif;
impl WifSection for Wif {
    const NAME: &str = sections::WIF;

    type Output = WifHeader;

    wr_fields! {
        version : "Version",
        date : "Date",
        developers : "Developers",
        source_program : "Source Program",
        ? source_version: "Source Version"
    }
}

pub(crate) struct ColorPalette;
impl WifSection for ColorPalette {
    const NAME: &str = sections::COLOR_PALETTE;

    type Output = super::ColorPalette;
    wr_fields! {entries: "Entries", range: "Range"}
}

pub(crate) struct WarpSymbolPalette;
impl WifSection for WarpSymbolPalette {
    const NAME: &str = sections::WARP_SYMBOL_PALETTE;

    type Output = super::WarpSymbolPalette;
    wr_fields! {entries: "Entries"}
}

pub(crate) struct WeftSymbolPalette;
impl WifSection for WeftSymbolPalette {
    const NAME: &str = sections::WEFT_SYMBOL_PALETTE;

    type Output = super::WarpSymbolPalette;

    wr_fields! {entries: "Entries"}
}

pub(crate) struct ColorTable;
impl WifSection for ColorTable {
    const NAME: &str = sections::COLOR_TABLE;

    type Output = super::BTreeMap<u32, Color>;
    wr_table! {}
}

pub(crate) struct Text;
impl WifSection for Text {
    const NAME: &str = sections::TEXT;

    type Output = super::Text;

    wr_fields! {
            ? title: "Title",
            ? author: "Author",
            ? address: "Address",
            ? email: "EMail",
            ? telephone: "Telephone",
            ? fax: "Fax",
    }
}

pub(crate) struct Weaving;
impl WifSection for Weaving {
    const NAME: &str = sections::WEAVING;

    type Output = super::Weaving;

    wr_fields! {
            shafts: "Shafts",
            treadles: "Treadles",
            ? rising_shed: "Rising Shed",
    }
}

pub(crate) struct Warp;
impl WifSection for Warp {
    const NAME: &str = sections::WARP;

    type Output = super::WarpS;

    wr_fields! {
            threads: "Threads",
            ? color: "Color",
            ? symbol: "Symbol",
            ? symbol_number: "Symbol Number",
            ? units: "Units",
            ? spacing: "Spacing",
            ? thickness: "Thickness",
            ? spacing_zoom: "Spacing Thickness",
            ? thickness_zoom: "Thickness Zoom",
    }
}

pub(crate) struct Weft;
impl WifSection for Weft {
    const NAME: &str = sections::WEFT;

    type Output = super::WeftS;

    wr_fields! {
            threads: "Threads",
            ? color: "Color",
            ? symbol: "Symbol",
            ? symbol_number: "Symbol Number",
            ? units: "Units",
            ? spacing: "Spacing",
            ? thickness: "Thickness",
            ? spacing_zoom: "Spacing Thickness",
            ? thickness_zoom: "Thickness Zoom",
    }
}

pub(crate) struct Notes;
impl WifSection for Notes {
    const NAME: &str = sections::NOTES;

    type Output = super::BTreeMap<u32, String>;
    wr_table! {}
}

pub(crate) struct Tieup;
impl WifSection for Tieup {
    const NAME: &str = sections::TIEUP;

    type Output = super::BTreeMap<Treadle, BTreeSet<Shaft>>;
    wr_table! {}
}

pub(crate) struct WarpSymbolTable;
impl WifSection for WarpSymbolTable {
    const NAME: &str = sections::WARP_SYMBOL_TABLE;

    type Output = super::BTreeMap<u32, String>;
    wr_table! {}
}

pub(crate) struct WeftSymbolTable;
impl WifSection for WeftSymbolTable {
    const NAME: &str = sections::WEFT_SYMBOL_TABLE;

    type Output = super::BTreeMap<u32, String>;
    wr_table! {}
}

pub(crate) struct Threading;
impl WifSection for Threading {
    const NAME: &str = sections::THREADING;

    type Output = super::BTreeMap<super::Warp, BTreeSet<Shaft>>;
    wr_table! {}
}

pub(crate) struct WarpThickness;
impl WifSection for WarpThickness {
    const NAME: &str = sections::WARP_THICKNESS;

    type Output = super::BTreeMap<super::Warp, f64>;
    wr_table! {}
}

pub(crate) struct WarpThicknessZoom;
impl WifSection for WarpThicknessZoom {
    const NAME: &str = sections::WARP_THICKNESS_ZOOM;

    type Output = super::BTreeMap<super::Warp, u32>;
    wr_table! {}
}

pub(crate) struct WarpSpacing;
impl WifSection for WarpSpacing {
    const NAME: &str = sections::WARP_SPACING;

    type Output = super::BTreeMap<super::Warp, f64>;
    wr_table! {}
}

pub(crate) struct WarpSpacingZoom;
impl WifSection for WarpSpacingZoom {
    const NAME: &str = sections::WARP_SPACING_ZOOM;

    type Output = super::BTreeMap<super::Warp, u32>;
    wr_table! {}
}

pub(crate) struct WarpColors;
impl WifSection for WarpColors {
    const NAME: &str = sections::WARP_COLORS;

    type Output = super::BTreeMap<super::Warp, u32>;
    wr_table! {}
}

pub(crate) struct WarpSymbols;
impl WifSection for WarpSymbols {
    const NAME: &str = sections::WARP_SYMBOLS;

    type Output = super::BTreeMap<super::Warp, u32>;
    wr_table! {}
}

pub(crate) struct WeftThickness;
impl WifSection for WeftThickness {
    const NAME: &str = sections::WEFT_THICKNESS;

    type Output = super::BTreeMap<super::Weft, f64>;
    wr_table! {}
}

pub(crate) struct WeftThicknessZoom;
impl WifSection for WeftThicknessZoom {
    const NAME: &str = sections::WEFT_THICKNESS_ZOOM;

    type Output = super::BTreeMap<super::Weft, u32>;
    wr_table! {}
}

pub(crate) struct WeftSpacing;
impl WifSection for WeftSpacing {
    const NAME: &str = sections::WEFT_SPACING;

    type Output = super::BTreeMap<super::Weft, f64>;
    wr_table! {}
}

pub(crate) struct WeftSpacingZoom;
impl WifSection for WeftSpacingZoom {
    const NAME: &str = sections::WEFT_SPACING_ZOOM;

    type Output = super::BTreeMap<super::Weft, u32>;
    wr_table! {}
}

pub(crate) struct WeftColors;
impl WifSection for WeftColors {
    const NAME: &str = sections::WEFT_COLORS;

    type Output = super::BTreeMap<super::Weft, u32>;
    wr_table! {}
}

pub(crate) struct WeftSymbols;
impl WifSection for WeftSymbols {
    const NAME: &str = sections::WEFT_SYMBOLS;

    type Output = super::BTreeMap<super::Weft, u32>;
    wr_table! {}
}

pub(crate) struct Treadling;
impl WifSection for Treadling {
    const NAME: &str = sections::TREADLING;

    type Output = super::Table<super::Weft, BTreeSet<Treadle>>;
    wr_table! {}
}

pub(crate) struct Liftplan;
impl WifSection for Liftplan {
    const NAME: &str = sections::LIFTPLAN;

    type Output = super::BTreeMap<super::Weft, BTreeSet<Shaft>>;
    wr_table! {}
}

fn parse_table<S, T>(ini: &Ini, section_name: &str) -> super::Result<Table<S, T>>
where
    S: FromStr + Ord,
    T: WifParse,
{
    let mut rv = Table::new();
    let section = ini
        .get_map_ref()
        .get(&section_name.to_lowercase())
        .ok_or_else(|| WifError::MissingSection {
            section: section_name.into(),
        })?;
    for (k, v) in section {
        let Some(v) = v else {
            continue;
        };
        let id: S = k.parse().map_err(|_| WifError::CouldNotParseTableKey {
            section: section_name.into(),
            key: k.clone(),
        })?;
        rv.insert(id, T::parse(v.clone()).add_context(section_name, k)?);
    }
    Ok(rv)
}
