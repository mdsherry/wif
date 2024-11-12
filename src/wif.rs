use std::collections::{BTreeMap, BTreeSet};

mod wif_sections;

#[cfg(test)]
mod tests;

use chrono::NaiveDate;
use configparser::ini::Ini;
use wif_sections::WifSection;

use crate::{
    liftplan_from_threading_and_treadle, wifparse::WifParse, Color, Result, Shaft, Table, Treadle,
    Warp, WarpOrWeft, Weft, WifContext, WifError,
};

#[derive(Debug, Clone)]
pub struct Wif {
    pub wif_header: WifHeader,
    pub color_palette: Option<ColorPalette>,
    pub warp_symbol_palette: Option<WarpSymbolPalette>,
    pub weft_symbol_palette: Option<WarpSymbolPalette>,
    pub text: Option<Text>,
    pub weaving: Option<Weaving>,
    pub warp: Option<WarpS>,
    pub weft: Option<WeftS>,
    pub color_table: Option<Table<u32, Color>>,
    pub notes: Option<Table<u32, String>>,
    pub tieup: Option<Table<Treadle, BTreeSet<Shaft>>>,
    pub warp_symbol_table: Option<Table<u32, String>>,
    pub weft_symbols_table: Option<Table<u32, String>>,
    pub threading: Option<Table<Warp, BTreeSet<Shaft>>>,
    pub warp_thickness: Option<Table<Warp, f64>>,
    pub warp_thickness_zoom: Option<Table<Warp, u32>>,
    pub warp_spacing: Option<Table<Warp, f64>>,
    pub warp_spacing_zoom: Option<Table<Warp, u32>>,
    pub warp_colors: Option<Table<Warp, u32>>,
    pub warp_symbols: Option<Table<Warp, u32>>,
    pub treadling: Option<Table<Weft, BTreeSet<Treadle>>>,
    pub liftplan: Option<Table<Weft, BTreeSet<Shaft>>>,
    pub weft_thickness: Option<Table<Weft, f64>>,
    pub weft_thickness_zoom: Option<Table<Weft, u32>>,
    pub weft_spacing: Option<Table<Weft, f64>>,
    pub weft_spacing_zoom: Option<Table<Weft, u32>>,
    pub weft_colors: Option<Table<Weft, u32>>,
    pub weft_symbols: Option<Table<Weft, u32>>,
    // Private code regions go here
}

impl Wif {
    pub fn shafts(&self) -> Option<u32> {
        self.weaving.as_ref().map(|w| w.shafts)
    }
    pub fn treadles(&self) -> Option<u32> {
        self.weaving.as_ref().map(|w| w.treadles)
    }
    pub fn width(&self) -> Option<u32> {
        self.warp.as_ref().map(|w| w.threads)
    }
    pub fn height(&self) -> Option<u32> {
        self.weft.as_ref().map(|w| w.threads)
    }

    pub fn build_or_validate_liftplan(&mut self) -> Result<()> {
        let liftplan =
            liftplan_from_threading_and_treadle(self.treadling.as_ref(), self.tieup.as_ref());
        match (liftplan, self.liftplan.as_ref()) {
            // Good luck weaving anything!
            (None, None) => Ok(()),
            // The pattern provided a lift plan instead of treadling
            (None, Some(_)) => Ok(()),
            // Fill in missing lift plan
            (Some(liftplan), None) => {
                self.liftplan = Some(liftplan);
                Ok(())
            }
            // Check for validity
            (Some(new_liftplan), Some(old_liftplan)) if &new_liftplan == old_liftplan => Ok(()),
            _ => Err(WifError::LiftPlanDoesNotMatchTreadling),
        }
    }

    fn get_ct(&self, color_idx: u32) -> Option<Color> {
        self.color_table
            .as_ref()
            .and_then(|ct| ct.get(&color_idx))
            .copied()
    }

    fn get_default_weft_color(&self) -> Option<Color> {
        self.get_ct(self.weft.as_ref()?.color?.idx)
    }

    pub fn weft_color(&self, weft: impl Into<Weft>) -> Option<Color> {
        let weft = weft.into();
        self.weft_colors
            .as_ref()
            .and_then(|wc| wc.get(&weft))
            .and_then(|idx| self.get_ct(*idx))
            .or_else(|| self.get_default_weft_color())
    }

    pub fn weft_color_u8(&self, weft: impl Into<Weft>) -> Option<[u8; 3]> {
        let range = self
            .color_palette
            .as_ref()
            .map(|cp| cp.range)
            .unwrap_or((0, 999));
        let convert =
            |old_value| ((old_value - range.0) as f64 / (range.1 - range.0) as f64 * 255.) as u8;
        self.weft_color(weft.into()).map(|color| {
            [
                convert(color.red),
                convert(color.green),
                convert(color.blue),
            ]
        })
    }

    fn get_default_warp_color(&self) -> Option<Color> {
        self.get_ct(self.warp.as_ref()?.color?.idx)
    }

    pub fn warp_color(&self, warp: impl Into<Warp>) -> Option<Color> {
        let warp = warp.into();
        self.warp_colors
            .as_ref()
            .and_then(|wc| wc.get(&warp))
            .and_then(|idx| self.get_ct(*idx))
            .or_else(|| self.get_default_warp_color())
    }

    pub fn warp_color_u8(&self, warp: impl Into<Warp>) -> Option<[u8; 3]> {
        let range = self
            .color_palette
            .as_ref()
            .map(|cp| cp.range)
            .unwrap_or((0, 999));
        let convert =
            |old_value| ((old_value - range.0) as f64 / (range.1 - range.0) as f64 * 255.) as u8;
        self.warp_color(warp.into()).map(|color| {
            [
                convert(color.red),
                convert(color.green),
                convert(color.blue),
            ]
        })
    }

    pub fn warp_or_weft(&self, warp: impl Into<Warp>, weft: impl Into<Weft>) -> Option<WarpOrWeft> {
        let warp = warp.into();
        let weft = weft.into();
        let liftplan = self.liftplan.as_ref()?;
        let threading = self.threading.as_ref()?;
        if let Some(shafts) = liftplan.get(&weft) {
            if let Some(thread_shafts) = threading.get(&warp) {
                if shafts.intersection(thread_shafts).next().is_some() {
                    Some(WarpOrWeft::Warp)
                } else {
                    Some(WarpOrWeft::Weft)
                }
            } else {
                Some(WarpOrWeft::Weft)
            }
        } else {
            // TODO: Check if the loom is rising shed or not
            Some(WarpOrWeft::Weft)
        }
    }
    pub fn write<W>(&self, output: &mut W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        let mut ini = configparser::ini::Ini::new_cs();

        wif_sections::Wif::write(&self.wif_header, &mut ini);

        macro_rules! write_section {
            ($($field:ident : $section:ident),*) => {
                $(
                    if let Some($field) = &self.$field {
                        wif_sections::$section::write($field, &mut ini);
                    }
                )*
            }
        }
        write_section! {
            color_palette: ColorPalette,
            warp_symbol_palette: WarpSymbolPalette,
            weft_symbol_palette: WeftSymbolPalette,
            color_table: ColorTable,
            text: Text,
            weaving: Weaving,
            warp: Warp,
            weft: Weft,
            notes: Notes,
            tieup: Tieup,
            warp_symbol_table: WarpSymbolTable,
            weft_symbols_table: WeftSymbolTable,
            threading: Threading,

            warp_thickness: WarpThickness,
            warp_thickness_zoom: WarpThicknessZoom,
            warp_spacing: WarpSpacing,
            warp_spacing_zoom: WarpSpacingZoom,
            warp_colors: WarpColors,
            warp_symbols: WarpSymbols,

            weft_thickness: WeftThickness,
            weft_thickness_zoom: WeftThicknessZoom,
            weft_spacing: WeftSpacing,
            weft_spacing_zoom: WeftSpacingZoom,
            weft_colors: WeftColors,
            weft_symbols: WeftSymbols,

            treadling: Treadling,
            liftplan: Liftplan
        }
        output.write_all(ini.writes().as_bytes())?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct WifHeader {
    pub version: String,
    pub date: NaiveDate,
    pub developers: String,
    pub source_program: String,
    pub source_version: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ColorPalette {
    pub entries: usize,
    pub range: (u32, u32),
}

#[derive(Debug, Clone)]
pub struct WarpSymbolPalette {
    pub entries: usize,
}

#[derive(Debug, Clone)]
pub struct Text {
    pub title: Option<String>,
    pub author: Option<String>,
    pub address: Option<String>,
    pub email: Option<String>,
    pub telephone: Option<String>,
    pub fax: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Weaving {
    pub shafts: u32,
    pub treadles: u32,
    pub rising_shed: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct WarpS {
    pub threads: u32,
    pub color: Option<BaseColor>,
    pub symbol: Option<String>,
    pub symbol_number: Option<usize>,
    pub units: Option<String>,
    pub spacing: Option<f64>,
    pub thickness: Option<f64>,
    pub spacing_zoom: Option<u32>,
    pub thickness_zoom: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct WeftS {
    pub threads: u32,
    pub color: Option<BaseColor>,
    pub symbol: Option<String>,
    pub symbol_number: Option<usize>,
    pub units: Option<String>,
    pub spacing: Option<f64>,
    pub thickness: Option<f64>,
    pub spacing_zoom: Option<u32>,
    pub thickness_zoom: Option<u32>,
}

fn get_field<T>(ini: &Ini, section: &str, field: &str) -> Result<Option<T>>
where
    T: WifParse,
{
    ini.get(section, field)
        .map(T::parse)
        .transpose()
        .add_context(section, field)
}

fn get_required_field<T>(ini: &Ini, section: &str, field: &str) -> Result<T>
where
    T: WifParse,
{
    ini.get(section, field)
        .map(T::parse)
        .transpose()
        .add_context(section, field)?
        .ok_or_else(|| WifError::MissingRequiredField {
            section: section.into(),
            field: field.into(),
        })
}

#[derive(Debug, Clone, Copy)]
pub struct BaseColor {
    pub idx: u32,
    pub alt: Option<Color>,
}

fn parse_base_color_opt(ini: &Ini, section: &str, field: &str) -> Result<Option<BaseColor>> {
    let mut s = ini.get(section, field);
    s.map(|s| {
        if s.contains(',') {
            todo!()
        } else {
            Ok(BaseColor {
                idx: u32::parse(s)?,
                alt: None,
            })
        }
    })
    .transpose()
}

fn parse_symbol_opt(ini: &Ini, section: &str, field: &str) -> Result<Option<String>> {
    Ok(ini.get(section, field))
}

pub fn parse(s: &str) -> Result<Wif, WifError> {
    let mut ini = configparser::ini::Ini::new();
    ini.read(s.into());
    macro_rules! read_section {
        ($name:ident) => {
            if has_section(&ini, wif_sections::$name::NAME)? {
                Some(wif_sections::$name::read(&ini)?)
            } else {
                None
            }
        };
    }
    let wif_header = wif_sections::Wif::read(&ini)?;
    let color_palette = read_section!(ColorPalette);
    let color_table = read_section!(ColorTable);
    let warp_symbol_palette = read_section!(WarpSymbolPalette);
    let weft_symbol_palette = read_section!(WeftSymbolPalette);
    let text = read_section!(Text);
    let weaving = read_section!(Weaving);
    let warp = read_section!(Warp);
    let weft = read_section!(Weft);
    let notes = read_section!(Notes);
    let tieup = read_section!(Tieup);
    let warp_symbol_table = read_section!(WarpSymbolTable);
    let weft_symbols_table = read_section!(WeftSymbolTable);
    let threading = read_section!(Threading);
    let warp_thickness = read_section!(WarpThickness);
    let warp_thickness_zoom = read_section!(WarpThicknessZoom);
    let warp_spacing = read_section!(WarpSpacing);
    let warp_spacing_zoom = read_section!(WarpSpacingZoom);
    let warp_colors = read_section!(WarpColors);
    let warp_symbols = read_section!(WarpSymbols);

    let weft_thickness = read_section!(WeftThickness);
    let weft_thickness_zoom = read_section!(WeftThicknessZoom);
    let weft_spacing = read_section!(WeftSpacing);
    let weft_spacing_zoom = read_section!(WeftSpacingZoom);
    let weft_colors = read_section!(WeftColors);
    let weft_symbols = read_section!(WeftSymbols);

    let treadling = read_section!(Treadling);
    let liftplan = read_section!(Liftplan);

    let mut wif = Wif {
        wif_header,
        color_palette,
        warp_symbol_palette,
        color_table,
        weft_symbol_palette,
        text,
        weaving,
        warp,
        weft,
        notes,
        tieup,
        // color_table: None,
        warp_symbol_table,
        weft_symbols_table,
        threading,

        warp_thickness,
        warp_thickness_zoom,
        warp_spacing,
        warp_spacing_zoom,
        warp_colors,
        warp_symbols,

        treadling,
        liftplan,

        weft_thickness,
        weft_thickness_zoom,
        weft_spacing,
        weft_spacing_zoom,
        weft_colors,
        weft_symbols,
    };
    wif.build_or_validate_liftplan()?;
    Ok(wif)
}

fn has_section(ini: &Ini, section_name: &str) -> Result<bool, WifError> {
    Ok(get_field(ini, "CONTENTS", section_name)?.unwrap_or(false))
}

pub mod sections {
    pub const CONTENTS: &str = "CONTENTS";
    pub const WIF: &str = "WIF";
    pub const COLOR_PALETTE: &str = "COLOR PALETTE";
    pub const COLOR_TABLE: &str = "COLOR TABLE";
    pub const WARP_SYMBOL_PALETTE: &str = "WARP SYMBOL PALETTE";
    pub const WEFT_SYMBOL_PALETTE: &str = "WEFT SYMBOL PALETTE";
    pub const TEXT: &str = "TEXT";
    pub const WEAVING: &str = "WEAVING";
    pub const WARP: &str = "WARP";
    pub const WEFT: &str = "WEFT";
    pub const NOTES: &str = "NOTES";
    pub const TIEUP: &str = "TIEUP";
    pub const WARP_SYMBOL_TABLE: &str = "WARP SYMBOL TABLE";
    pub const WEFT_SYMBOL_TABLE: &str = "WEFT SYMBOL TABLE";
    pub const THREADING: &str = "THREADING";
    pub const WARP_THICKNESS: &str = "WARP THICKNESS";
    pub const WARP_THICKNESS_ZOOM: &str = "WARP THICKNESS ZOOM";
    pub const WARP_SPACING: &str = "WARP SPACING";
    pub const WARP_SPACING_ZOOM: &str = "WARP SPACING ZOOM";
    pub const WARP_COLORS: &str = "WARP COLORS";
    pub const WARP_SYMBOLS: &str = "WARP SYMBOLS";
    pub const TREADLING: &str = "TREADLING";
    pub const LIFTPLAN: &str = "LIFTPLAN";
    pub const WEFT_THICKNESS: &str = "WEFT THICKNESS";
    pub const WEFT_THICKNESS_ZOOM: &str = "WEFT THICKNESS ZOOM";
    pub const WEFT_SPACING: &str = "WEFT SPACING";
    pub const WEFT_SPACING_ZOOM: &str = "WEFT SPACING ZOOM";
    pub const WEFT_COLORS: &str = "WEFT COLORS";
    pub const WEFT_SYMBOLS: &str = "WEFT SYMBOLS";
}

pub enum Symbol {
    Char(char),
    Quoted(char),
    Code(char),
}

struct Section<'a> {
    ini: &'a mut Ini,
    name: String,
}
impl<'a> Section<'a> {
    fn new(ini: &'a mut Ini, name: impl Into<String>) -> Self {
        Self {
            ini,
            name: name.into(),
        }
    }
    fn record_usage(&mut self) {
        self.ini
            .set(sections::CONTENTS, &self.name, Some("true".into()));
    }
    fn write<W: WifParse>(&mut self, key: &str, value: &W) {
        let val = value.unparse();
        if val.is_some() {
            self.ini.set(&self.name, key, val);
        }
    }
    fn write_table<T, W>(&mut self, table: &BTreeMap<T, W>)
    where
        T: std::fmt::Display,
        W: WifParse,
    {
        for (k, v) in table {
            let k_str = k.to_string();
            self.write(&k_str, v);
        }
    }
}
