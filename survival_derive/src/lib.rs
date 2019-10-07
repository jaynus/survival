#![deny(clippy::pedantic, clippy::all)]
#![allow(clippy::default_trait_access, clippy::module_name_repetitions)]

extern crate proc_macro;

use crate::proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream, Result},
    Path,
};

fn impl_named_definition(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl Named for #name {
            fn name(&self) -> &str {
                self.name.as_str()
            }
            fn id(&self) -> Option<u32> {
                self.id
            }
            fn set_id(&mut self, id: u32) {
                self.id = Some(id);
            }
        }
        impl Definition for #name {}
    };
    gen.into()
}

#[proc_macro_derive(NamedDefinition)]
pub fn named_definition_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_named_definition(&ast)
}

struct DefAttribute {
    storage: Path,
}

impl Parse for DefAttribute {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        let _parenthesized_token = syn::parenthesized!(content in input);

        Ok(Self {
            storage: content.parse()?,
        })
    }
}

fn impl_definition_component(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let def = ast
        .attrs
        .iter()
        .find_map(|attr| {
            if attr.path.segments[0].ident == "def" {
                Some(
                    syn::parse2::<DefAttribute>(attr.tts.clone())
                        .unwrap()
                        .storage,
                )
            } else {
                None
            }
        })
        .unwrap();

    let gen = quote! {
        impl #impl_generics DefinitionComponent for #name #ty_generics #where_clause {
            type DefinitionType = #def;
            fn fetch_def<'a>(
                &self,
                storage: &'a DefinitionStorage<Self::DefinitionType>,
            ) -> Option<&'a Self::DefinitionType> {
                use std::convert::TryInto;
                storage.get(self.def.try_into().unwrap())
            }
        }
    };

    gen.into()
}

#[proc_macro_derive(DefinitionComponent, attributes(def))]
pub fn definition_component_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_definition_component(&ast)
}
