#![warn(clippy::all)]
#![deny(missing_docs)]
//! A macro to help parse responses from 3scale Porta.
//!
//! Usage:
//!
//! ```example
//! #[derive(Serialize, Deserialize)]
//! #[straitjacket]
//! struct PortaModel {
//!     // the model's fields, you can use serde attributes here
//! }
//! ```
//!
//! This will generate a set of structures that include implementations for `serde`'s
//! `Serializable` and `Deserializable` traits which will be helpful when parsing the
//! quirky responses for collections returned by Porta.
//!
//! # Dependencies
//!
//! You are required to provide a deserializable type in scope to parse metadata, by
//! default referred to simply as `Metadata` (but this is customizable).
//!
//! # Troubleshooting
//!
//! If you find errors such as `E0412: cannot find type ... in this scope` or error
//! `E0282: type annotations needed`, make sure you have a `Metadata` type or you
//! specify its correct name as an attribute parameter to the macro.
//!
//! If parsing does not work for you, make sure to use the attribute parameters to
//! ensure the actual names used by Porta match with the generated code.
//!
//! ```example
//! #[derive(Deserialize)]
//! struct PlanMetadata {
//!     // Plan's metadata as returned by 3scale
//! }
//!
//! // *Note*: returned data uses `application_plan` for collection items rather
//! //         than `plan`.
//! #[derive(Serialize, Deserialize)]
//! #[straitjacket(name_snake = "application_plan", metadata = "PlanMetadata")]
//! struct Plan {
//!     // typical Plan data
//! }
//! ```
//!
//! Ultimately you can take a look at the generated code via tools like cargo expand
//! and see debugging info for this crate via the `macro-debug` feature.
//!

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

use std::iter::FromIterator;

#[cfg(feature = "macro-debug")]
macro_rules! macro_debug {
    ( $($e:expr),* ) => {
        println!($($e),*)
    }
}

#[cfg(not(feature = "macro-debug"))]
macro_rules! macro_debug {
    ( $($e:expr),* ) => {
        ()
    };
}

mod sj {
    use proc_macro2::Ident;

    #[derive(Debug, Clone)]
    pub struct StraitJacket {
        name: Ident,
        name_snake: Ident,
        name_and_metadata: Ident,
        name_tag: Ident,
        plural: Ident,
        plural_snake: Ident,
        metadata: Ident,
    }

    macro_rules! getter {
        ( $id:ident ) => {
            pub fn $id(&self) -> &Ident {
                &self.$id
            }
        };
    }

    impl StraitJacket {
        getter!(name);
        getter!(name_snake);
        getter!(name_and_metadata);
        getter!(name_tag);
        getter!(plural);
        getter!(plural_snake);
        getter!(metadata);

        pub fn new(
            name: Ident,
            name_snake: Ident,
            name_and_metadata: Ident,
            name_tag: Ident,
            plural: Ident,
            plural_snake: Ident,
            metadata: Ident,
        ) -> Self {
            Self {
                name,
                name_snake,
                name_and_metadata,
                name_tag,
                plural,
                plural_snake,
                metadata,
            }
        }
    }

    impl From<super::builder::StraitJacketBuilder> for StraitJacket {
        fn from(sb: super::builder::StraitJacketBuilder) -> Self {
            sb.build()
        }
    }
}

use sj::StraitJacket;

mod builder {
    use super::StraitJacket;
    use inflector::Inflector;
    use proc_macro2::{Ident, Span};

    macro_rules! attribute {
        ( $id:ident ) => {
            pub fn $id(mut self, value: &str) -> Self {
                let _ = self.$id.replace(Ident::new(value, Span::call_site()));
                self
            }
        };
        ( $id:ident, $getter:ident ) => {
            pub fn $getter(&self) -> Option<Ident> {
                self.$id.as_ref()
            }

            pub fn $id(mut self, value: &str) -> Self {
                let _ = self.$id.replace(Ident::new(value, Span::call_site()));
                self
            }
        };
    }

    #[derive(Debug, Clone)]
    pub struct StraitJacketBuilder {
        name: Ident,
        name_snake: Option<Ident>,
        name_and_metadata: Option<Ident>,
        name_tag: Option<Ident>,
        plural: Option<Ident>,
        plural_snake: Option<Ident>,
        metadata: Option<Ident>,
    }

    impl StraitJacketBuilder {
        pub fn new(name: Ident) -> Self {
            Self {
                name,
                name_snake: None,
                name_and_metadata: None,
                name_tag: None,
                plural: None,
                plural_snake: None,
                metadata: None,
            }
        }

        attribute!(name_snake);
        attribute!(name_and_metadata);
        attribute!(name_tag);
        attribute!(plural);
        attribute!(plural_snake);
        attribute!(metadata);

        pub fn set(self, field: &str, value: &str) -> Self {
            match field {
                "name_snake" => self.name_snake(value),
                "name_and_metadata" => self.name_and_metadata(value),
                "name_tag" => self.name_tag(value),
                "plural" => self.plural(value),
                "plural_snake" => self.plural_snake(value),
                "metadata" => self.metadata(value),
                _ => {
                    macro_debug!("unknown attribute {:#?}", field);
                    self
                }
            }
        }

        pub fn build(self) -> StraitJacket {
            use quote::format_ident;

            let name_s = self.name.to_string();
            let plural = name_s.to_plural();

            StraitJacket::new(
                self.name,
                self.name_snake.unwrap_or_else(|| {
                    Ident::new(name_s.to_snake_case().as_str(), Span::call_site())
                }),
                self.name_and_metadata
                    .unwrap_or_else(|| format_ident!("{}AndMetadata", name_s)),
                self.name_tag
                    .unwrap_or_else(|| format_ident!("{}Tag", name_s)),
                self.plural
                    .unwrap_or_else(|| Ident::new(plural.as_str(), Span::call_site())),
                self.plural_snake.unwrap_or_else(|| {
                    Ident::new(plural.to_snake_case().as_str(), Span::call_site())
                }),
                self.metadata
                    .unwrap_or_else(|| Ident::new("Metadata", Span::call_site())),
            )
        }
    }
}

mod parser {
    use syn::{Ident, Lit, MetaNameValue, NestedMeta};

    fn get_key_value(mnv: &MetaNameValue) -> Option<(&Ident, &Lit)> {
        macro_debug!("Meta(NameValue(mnv)): {:#?}", mnv);
        match mnv {
            syn::MetaNameValue {
                lit: Lit::Str(_lit_str),
                ..
            } => match mnv.path.get_ident() {
                Some(ident) => {
                    macro_debug!(
                        "Found attribute {} = {}",
                        ident.to_string(),
                        _lit_str.value()
                    );
                    Some((ident, &mnv.lit))
                }
                None => {
                    macro_debug!("Found string literal value {} but no suitable attribute name for path {:#?}", _lit_str.value(), mnv.path);
                    None
                }
            },
            syn::MetaNameValue {
                lit: _lit,
                path: _path,
                ..
            } => {
                macro_debug!(
                    "Found non string literal value {:#?} for path {:#?}",
                    _lit,
                    _path
                );
                None
            }
        }
    }

    pub fn get_attributes_and_values(
        nestedmetas: &[NestedMeta],
    ) -> impl Iterator<Item = (&Ident, &Lit)> {
        nestedmetas.iter().filter_map(|nestedmeta| {
            use syn::Meta::*;

            match nestedmeta {
                NestedMeta::Meta(NameValue(mnv)) => get_key_value(mnv),
                _other => {
                    macro_debug!("Unhandled NestedMeta: {:#?}", _other);
                    None
                }
            }
        })
    }
}

/// The `straitjacket` macro.
///
/// This macro should be applied to structures modelling a Porta resource.
///
/// It supports specifying attributes to customize the generated code. It is
/// of particular importance to specify the plural and/or snake case forms
/// of the resource if they aren't derived correctly.
///
/// The following set of attributes are accepted to customize the output:
///
/// - `name_snake`: How the model's snake case is represented by Porta.
/// - `plural`: The plural form of the model. If unspecified a best effort will be used.
/// - `plural_snake`: The snake case form of the plural used in Porta responses.
/// - `metadata`: The name of the type to add as metadata for this resource. Note that
///             this type must be provided by the user, since it depends on the resource.
///
/// The following set of attributes are considered internal, since you don't need
/// to reference them directly, but you can still work with them and could potentially
/// cause name clashes:
///
/// - `name_and_metadata`: The name of the type used to deserialize a resource along its
///                        metadata (ie. link references, timestamps, etc)
/// - `name_tag`: The name of the type used to match on the quirky tags Porta uses.
#[proc_macro_attribute]
pub fn straitjacket(attr: TokenStream, item: TokenStream) -> TokenStream {
    macro_debug!("attributes: {}", attr);
    macro_debug!("item: {}", item);

    // `item` is consumed by the parsing, but we need to reproduce it verbatim
    // so we clone it here for usage later on.
    let c = item.clone();

    // parse the attributes and the item this macro applies to into ASTs
    let attr_ast = parse_macro_input!(attr as syn::AttributeArgs);
    let item_ast = parse_macro_input!(item as DeriveInput);

    // the item's name (ie. the struct name)
    let name = item_ast.ident;

    // a helper structu to validate the attributes and/or provide defaults
    let mut sjbuilder = builder::StraitJacketBuilder::new(name);

    // parse attributes
    for (ident, lit) in parser::get_attributes_and_values(&attr_ast) {
        sjbuilder = match (ident.to_string().as_str(), lit) {
            (key, syn::Lit::Str(lit_str)) => sjbuilder.set(key, lit_str.value().as_str()),
            _ => sjbuilder,
        };
    }

    // get the final configuration
    let sj = sjbuilder.build();

    // the `quote` macro requires in-scope local bindings
    let name = sj.name();
    let name_snake = sj.name_snake();
    let name_and_metadata = sj.name_and_metadata();
    let name_tag = sj.name_tag();
    let plural = sj.plural();
    let plural_snake = sj.plural_snake();
    let metadata = sj.metadata();
    let name_snake_s = name_snake.to_string();
    let plural_snake_s = plural_snake.to_string();

    // generate code
    let quoted_plural = quote! {
        #[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
        struct #name_and_metadata {
            #[serde(flatten)]
            item: #name,
            #[serde(flatten, skip_serializing)]
            metadata: Option<#metadata>,
        }
        #[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
        enum #name_tag {
            #[serde(rename = #name_snake_s)]
            Tag(#name_and_metadata),
        }
        #[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
        struct #plural {
            #[serde(rename = #plural_snake_s)]
            #plural_snake: Vec<#name_tag>,
        }

        impl From<Vec<#name>> for #plural {
            fn from(mrvec: Vec<#name>) -> Self {
                #plural {
                    #plural_snake: mrvec
                        .into_iter()
                        .map(|item| #name_tag::Tag(#name_and_metadata {
                            item,
                            metadata: None,
                        })).collect::<Vec<_>>(),
                }
            }
        }

        impl From<#plural> for Vec<#name_and_metadata> {
            fn from(mr: #plural) -> Self {
                mr.#plural_snake.into_iter().map(|mr| {
                    let #name_tag::Tag(mramd) = mr;
                    mramd
                }).collect()
            }
        }

        impl From<#plural> for Vec<#name> {
            fn from(mr: #plural) -> Self {
                mr.#plural_snake.into_iter().map(|mr| {
                    let #name_tag::Tag(mramd) = mr;
                    mramd.item
                }).collect()
            }
        }
    };

    // avoiding the Vec could be done via unstable std::array::IntoIter
    let q = vec![c, quoted_plural.into()];
    // emit the generated code
    TokenStream::from_iter(q.into_iter())
}
