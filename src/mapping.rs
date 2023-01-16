use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::path::{Path, PathBuf};
use anyhow::anyhow;
use proc_macro2::{Ident, Literal};
use quote::{format_ident, quote};
use syn::{AttrStyle, File, Item, ItemMod, Token};
use syn::parse::{Parse, ParseStream};

#[derive(Default)]
pub struct Mapping {
    items: HashMap<String, Mod>,
}

impl Display for Mapping {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.items
            .iter()
            .map(|(_, v)| v)
            .try_for_each(|m| write!(f, "{m}"))
    }
}

impl Mapping {
    pub fn cleanup(&mut self, dir: &Path) {
        self.items.retain(|key, _| {
            let mut path = PathBuf::from(dir);
            path.push(key);
            path.is_file()
        });
    }

    pub fn delete_file<'a>(&'a mut self, path: &'a str) -> Option<&str> {
        self.items.remove(path).map(|_| path)
    }

    pub fn insert_file(&mut self, file_name: &str) {
        let m = Mod::file_mod(file_name);
        self.items.insert(file_name.to_string(), m);
    }

    pub fn insert_leetcode_file(&mut self, file_name: &str) -> anyhow::Result<()> {
        let m = Mod::leetcode_mod(file_name)?;
        self.items.insert(file_name.to_string(), m);
        Ok(())
    }

    pub fn from_str(input: &str) -> Result<Self, syn::Error> {
        let file_syntax: File = syn::parse_str(input)?;

        let map = file_syntax
            .items
            .into_iter()
            .filter_map(|item| match item {
                Item::Mod(item_mod) => Some(item_mod),
                _ => None,
            })
            .map(Mod::from)
            .fold(HashMap::new(), |mut acc, item| {
                acc.insert(item.file_name(), item);
                acc
            });

        Ok(Mapping { items: map })
    }
}

#[derive(Clone)]
struct ModPath {
    _punct: Token![=],
    pub(crate) path: String,
}

impl Parse for ModPath {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            _punct: input.parse()?,
            path: input.call(Literal::parse).map(|lit| {
                lit.to_string()
                    .get(1..(lit.to_string().len() - 1))
                    .unwrap_or_default()
                    .to_string()
            })?,
        })
    }
}

impl From<ItemMod> for Mod {
    fn from(item_mode: ItemMod) -> Self {
        let attr: Option<ModPath> = item_mode
            .attrs
            .into_iter()
            .filter(|attr| matches!(attr.style, AttrStyle::Outer))
            .find_map(|attr| {
                attr.path
                    .get_ident()
                    .filter(|ident| *ident == "path")
                    .map(|_| attr.tokens)
            })
            .and_then(|ts| match syn::parse2(ts) {
                Ok(ret) => Some(ret),
                Err(e) => {
                    log::error!("parse mod ident fail, {e}");
                    None
                }
            });

        Self {
            attr: attr.map(|m| m.path),
            identity: item_mode.ident,
        }
    }
}

struct Mod {
    attr: Option<String>,
    identity: Ident,
}

impl Display for Mod {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(path) = &self.attr {
            let lit = Literal::string(path);
            let ts = quote! {
                #[path = #lit]
            };
            writeln!(f, "{ts}")?;
        }

        let identity = &self.identity;
        let ts = quote! {
            mod #identity;
        };

        writeln!(f, "{ts}")
    }
}

fn filter_numer<T>(input: T) -> anyhow::Result<u64>
where
    T: Deref<Target = str>,
{
    input
        .chars()
        .filter(|c| c.is_digit(10))
        .collect::<String>()
        .parse()
        .map_err(|_| anyhow!("invalid input: {}", &*input))
}

impl Mod {
    fn file_name(&self) -> String {
        self.attr
            .as_ref()
            .map(|s| s.clone())
            .unwrap_or_else(|| format!("{}.rs", self.identity).into())
    }

    fn file_mod(file_name: &str) -> Self {
        let prefix = file_name.rfind(".")
            .unwrap_or(file_name.len());
        Self {
            attr: None,
            identity: format_ident!("{}", file_name[0..prefix]),
        }
    }

    fn leetcode_mod(file_name: &str) -> anyhow::Result<Self> {
        let file_number = filter_numer(file_name)?;
        let identity = format_ident!("leetcode_{file_number}");

        Ok(Self {
            attr: Some(file_name.to_string()),
            identity,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_number() {
        assert_eq!(filter_numer("10.test.rs").unwrap(), 10);
    }

    #[test]
    fn test() {
        let test = r#"
        #[path="1.rs"]
        mod a;

        #[path="2.rs"]
        mod b;
        "#;

        let mapping = Mapping::from_str(test).unwrap();
        println!("{mapping}");
    }
}
