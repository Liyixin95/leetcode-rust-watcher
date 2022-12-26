use anyhow::anyhow;
use os_str_bytes::OsStrBytes;
use proc_macro2::{Ident, Literal, TokenStream};
use quote::{format_ident, quote, ToTokens};
use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::ops::Deref;
use std::path::{Path, PathBuf};
use syn::parse::{Parse, ParseStream};
use syn::{AttrStyle, File, Item, ItemMod, Token};

pub struct Mapping {
    items: HashMap<OsString, Mod>,
}

impl Mapping {
    pub fn cleanup(&mut self, dir: &Path) {
        self.items.retain(|key, _| {
            let mut path = PathBuf::from(dir);
            path.push(key);
            path.is_file()
        });
    }

    pub fn delete_file<'a>(&'a mut self, path: &'a OsStr) -> Option<&OsStr> {
        self.items.remove(path).map(|_| path)
    }

    pub fn insert_file(&mut self, file_path: PathBuf, file_name: &OsStr) -> anyhow::Result<()> {
        let m = Mod::new(file_path, file_name)?;
        self.items.insert(m.file_name(), m);
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

    pub fn print(&self) -> String {
        let iter = self.items.iter().map(|(_, v)| v);

        let tokenstream = quote! {
            #(#iter)*
        };

        tokenstream.to_string()
    }
}

#[derive(Clone)]
struct ModPath {
    _punct: Token![=],
    pub(crate) path: PathBuf,
}

impl Parse for ModPath {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            _punct: input.parse()?,
            path: input.call(Literal::parse).map(|lit| {
                let path = lit.to_string();
                let path = path
                    .get(1..(path.len() - 1))
                    .unwrap_or_default()
                    .to_string();
                PathBuf::from(path)
            })?,
        })
    }
}

impl ToTokens for Mod {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if let Some(path) = &self.attr {
            let lit = Literal::byte_string(path.as_os_str().to_raw_bytes().as_ref());
            let attr = quote! {
                #[path = #lit]
            };

            attr.to_tokens(tokens);
        }

        let identity = &self.identity;
        let tokenstream = quote! {
            mod #identity;
        };

        tokenstream.to_tokens(tokens);
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
    attr: Option<PathBuf>,
    identity: Ident,
}

fn filter_numer<T>(input: T) -> anyhow::Result<u64>
where
    T: Deref<Target = OsStr>,
{
    let lossy = input.to_string_lossy();

    lossy
        .chars()
        .filter(|c| c.is_digit(10))
        .collect::<String>()
        .parse()
        .map_err(|_| anyhow!("invalid input: {lossy}"))
}

impl Mod {
    fn file_name(&self) -> OsString {
        self.attr
            .as_ref()
            .and_then(|s| s.file_name())
            .map(|s| s.to_os_string())
            .unwrap_or_else(|| format!("{}.rs", self.identity).into())
    }

    fn new(file_path: PathBuf, file_name: &OsStr) -> anyhow::Result<Self> {
        let file_number = filter_numer(file_name)?;
        let identity = format_ident!("leetcode-{file_number}");

        Ok(Self {
            attr: Some(file_path),
            identity,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_number() {
        let input = OsString::from("10.test.rs");
        assert_eq!(filter_numer(input).unwrap(), 10);
    }

    #[test]
    fn test() {
        let test = r#"
        #[path="./1.rs"]
        mod a;

        #[path="./2.rs"]
        mod b;
        "#;

        let mapping = Mapping::from_str(test).unwrap();
        let ret = mapping.print();
        println!("{ret}");
    }

    #[test]
    fn test1() {
        let s = String::from("中文");
    }
}
