extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput};

#[proc_macro_derive(FromStr)]
pub fn from_str_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let expanded = match input.data {
        Data::Enum(data_enum) => {
            let variants = data_enum.variants.iter().map(|variant| {
                let variant_name = &variant.ident;
                let variant_str = variant_name.to_string();
                let variant_value = variant.discriminant.as_ref().map_or_else(
                    || quote! { #variant_name as isize },
                    |(_, expr)| quote! { #expr },
                );
                quote! {
                    #variant_str => Ok(#name::#variant_name),
                    s if s.parse::<isize>().ok() == Some(#variant_value) => Ok(#name::#variant_name),
                }
            });

            quote! {
                impl std::str::FromStr for #name {
                    type Err = anyhow::Error;

                    fn from_str(s: &str) -> Result<Self, Self::Err> {
                        match s {
                            #(#variants)*
                            _ => Err(anyhow::anyhow!("Invalid variant: {}", s)),
                        }
                    }
                }
            }
        }
        Data::Struct(_) => {
            quote! {
                impl std::str::FromStr for #name {
                    type Err = anyhow::Error;

                    fn from_str(s: &str) -> Result<Self, Self::Err> {
                        let id = s.parse()?;
                        Ok(Self(id))
                    }
                }
            }
        }
        _ => unimplemented!(),
    };

    TokenStream::from(expanded)
}
