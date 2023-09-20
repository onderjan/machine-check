use std::fmt::Display;

use anyhow::anyhow;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Sid(usize);

impl Sid {}

impl TryFrom<&str> for Sid {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if let Ok(sid) = value.parse() {
            Ok(Sid { 0: sid })
        } else {
            return Err(anyhow!("Cannot parse sid '{}'", value));
        }
    }
}

impl Display for Sid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Nid(usize);

impl Nid {
    pub fn create_ident(&self, flavor: &str) -> Ident {
        Ident::new(&format!("{}_{}", flavor, self.0), Span::call_site())
    }
}

impl TryFrom<&str> for Nid {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if let Ok(nid) = value.parse() {
            Ok(Nid { 0: nid })
        } else {
            return Err(anyhow!("Cannot parse nid '{}'", value));
        }
    }
}

impl Display for Nid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FlippableNid {
    pub flip: bool,
    pub nid: Nid,
}

impl TryFrom<&str> for FlippableNid {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let (flip, nid) = if value.starts_with("-") {
            (true, &value[1..])
        } else {
            (false, &value[..])
        };

        let nid = Nid::try_from(nid)?;

        Ok(FlippableNid { flip, nid: nid })
    }
}

impl Display for FlippableNid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let sign = if self.flip { "-" } else { "" };
        write!(f, "{}{}", sign, self.nid)
    }
}

impl FlippableNid {
    pub fn create_tokens(&self, flavor: &str) -> TokenStream {
        let ident = self.nid.create_ident(flavor);
        if self.flip {
            quote!((!#ident))
        } else {
            quote!(#ident)
        }
    }
}
