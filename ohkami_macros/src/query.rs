use proc_macro2::{TokenStream};
use quote::{quote, ToTokens};
use syn::{Result, parse_str, Type};

use crate::components::*;


#[allow(non_snake_case)]
pub(super) fn Query(data: TokenStream) -> Result<TokenStream> {
    let data = parse_struct("Queries", data)?;

    let impl_from_request = {
        let struct_name = &data.ident;
        let lifetimes = &data.generics; // checked to only contains lifetimes in `parse_struct`

        let fields = data.fields.iter().map(|f| {
            let field_name = f.ident.as_ref().unwrap(/* already checked in `parse_struct` */);
            let field_name_str = field_name.to_string();
            let field_type = &f.ty;
            let field_type_str = field_type.to_token_stream().to_string();

            if field_type_str.starts_with("Option") {
                let inner_type = parse_str::<Type>(field_type_str.strip_prefix("Option <").unwrap().strip_suffix(">").unwrap()).unwrap();
                quote!{
                    #field_name: req.query::<#inner_type>(#field_name_str)
                        .transpose()?,
                }
            } else {
                quote!{
                    #field_name: req.query::<#field_type>(#field_name_str) // Option<Result<_>>
                        .ok_or_else(|| ::std::borrow::Cow::Borrowed(
                            concat!("Expected query parameter `", #field_name_str, "`")
                        ))??,
                }
            } 
        });
        
        quote!{
            impl #lifetimes ::ohkami::FromRequest for #struct_name #lifetimes {
                fn parse(req: &::ohkami::Request) -> ::std::result::Result<Self, ::std::borrow::Cow<'static, str>> {
                    ::std::result::Result::Ok(Self {
                        #( #fields )*
                    })
                }
            }
        }
    };

    Ok(quote!{
        #data
        #impl_from_request
    })
}