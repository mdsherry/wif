use std::collections::BTreeSet;

use chrono::NaiveDate;

use crate::wif::BaseColor;
use crate::{Color, Shaft, Symbol, Treadle, Warp, Weft, WifError};

pub trait WifParse {
    fn parse(s: String) -> super::Result<Self>
    where
        Self: Sized;
    fn unparse(&self) -> Option<String>;
}
impl WifParse for NaiveDate {
    fn parse(s: String) -> crate::Result<Self>
    where
        Self: Sized,
    {
        NaiveDate::parse_from_str(&s, "%B %d, %Y").map_err(|e| WifError::InvalidDate { error: e })
    }

    fn unparse(&self) -> Option<String> {
        Some(self.format("%B %d, %Y").to_string())
    }
}
impl<T> WifParse for Option<T>
where
    T: WifParse,
{
    fn parse(s: String) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(Some(T::parse(s)?))
    }

    fn unparse(&self) -> Option<String> {
        self.as_ref().and_then(|v| v.unparse())
    }
}
impl WifParse for u32 {
    fn parse(s: String) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(s.parse()?)
    }

    fn unparse(&self) -> Option<String> {
        Some(self.to_string())
    }
}
impl WifParse for Shaft {
    fn parse(s: String) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(s.parse()?)
    }

    fn unparse(&self) -> Option<String> {
        Some(self.0.to_string())
    }
}
impl WifParse for Weft {
    fn parse(s: String) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(s.parse()?)
    }

    fn unparse(&self) -> Option<String> {
        Some(self.0.to_string())
    }
}
impl WifParse for Warp {
    fn parse(s: String) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(s.parse()?)
    }

    fn unparse(&self) -> Option<String> {
        Some(self.0.to_string())
    }
}
impl WifParse for Treadle {
    fn parse(s: String) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(s.parse()?)
    }

    fn unparse(&self) -> Option<String> {
        Some(self.0.to_string())
    }
}
impl WifParse for usize {
    fn parse(s: String) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(s.parse()?)
    }

    fn unparse(&self) -> Option<String> {
        Some(self.to_string())
    }
}

impl WifParse for f64 {
    fn parse(s: String) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(s.parse()?)
    }

    fn unparse(&self) -> Option<String> {
        Some(self.to_string())
    }
}

impl WifParse for String {
    fn parse(s: String) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(s)
    }

    fn unparse(&self) -> Option<String> {
        Some(self.clone())
    }
}

impl WifParse for bool {
    fn parse(s: String) -> crate::Result<Self>
    where
        Self: Sized,
    {
        match s.to_lowercase().as_str() {
            "true" | "on" | "yes" | "1" => Ok(true),
            "false" | "off" | "no" | "0" => Ok(false),
            _ => Err(WifError::ExpectedBool { saw: s.to_string() }),
        }
    }

    fn unparse(&self) -> Option<String> {
        Some(self.to_string())
    }
}

impl<T> WifParse for (T, T)
where
    T: WifParse,
{
    fn parse(s: String) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let (a, b) = s
            .split_once(',')
            .ok_or_else(|| WifError::ExpectedPair { saw: s.clone() })?;
        Ok((T::parse(a.into())?, T::parse(b.into())?))
    }

    fn unparse(&self) -> Option<String> {
        self.0.unparse().zip(self.1.unparse()).map(|(mut a, b)| {
            a += ",";
            a += &b;
            a
        })
    }
}

impl<T> WifParse for Vec<T>
where
    T: WifParse,
{
    fn parse(s: String) -> crate::Result<Self>
    where
        Self: Sized,
    {
        s.split(',').map(|s| T::parse(s.into())).collect()
    }

    fn unparse(&self) -> Option<String> {
        self.iter()
            .map(|v| v.unparse())
            .collect::<Option<Vec<String>>>()
            .map(|v| v.join(","))
    }
}

impl<T> WifParse for BTreeSet<T>
where
    T: WifParse + Ord,
{
    fn parse(s: String) -> crate::Result<Self>
    where
        Self: Sized,
    {
        s.split(',').map(|s| T::parse(s.into())).collect()
    }

    fn unparse(&self) -> Option<String> {
        self.iter()
            .map(|v| v.unparse())
            .collect::<Option<Vec<String>>>()
            .map(|v| v.join(","))
    }
}
impl WifParse for Color {
    fn parse(s: String) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let v: Vec<_> = s
            .split(',')
            .map(|s| s.trim().parse::<u32>())
            .collect::<Result<_, _>>()?;
        if v.len() != 3 {
            Err(WifError::ColorsMustBeThreeParts)
        } else {
            Ok(Color {
                red: v[0],
                green: v[1],
                blue: v[2],
            })
        }
    }

    fn unparse(&self) -> Option<String> {
        Some(format!("{},{},{}", self.red, self.green, self.blue))
    }
}
impl WifParse for BaseColor {
    fn parse(s: String) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(BaseColor {
            idx: u32::parse(s)?,
            alt: None,
        })
    }

    fn unparse(&self) -> Option<String> {
        self.idx.unparse()
    }
}

impl WifParse for Symbol {
    fn parse(s: String) -> crate::Result<Self>
    where
        Self: Sized,
    {
        if s.starts_with('\'') {
            Ok(Symbol::Quoted(s.chars().nth(1).unwrap()))
        } else if let Some(rest) = s.strip_prefix('#') {
            Ok(Symbol::Code(char::from_u32(rest.parse::<u32>()?).unwrap()))
        } else {
            Ok(Symbol::Char(s.chars().nth(0).unwrap()))
        }
    }

    fn unparse(&self) -> Option<String> {
        match self {
            Symbol::Char(c) => Some(c.to_string()),
            Symbol::Quoted(c) => Some(format!("\\{c}")),
            Symbol::Code(c) => Some(format!("#{}", *c as u32)),
        }
    }
}
