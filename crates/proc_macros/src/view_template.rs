use proc_macro::TokenStream;
use quote::quote;
use syn::parse::Parser;
use syn::{Field, Fields, ItemStruct};

pub fn impl_attr_web_view_template(_args: TokenStream, mut ast: ItemStruct) -> TokenStream {
    if let Fields::Named(ref mut fields) = ast.fields {
        fields.named.push({
            Field::parse_named
                .parse2({
                    quote! {
                       pub _base: ::web_core::view_template::ViewTemplateBase
                    }
                })
                .unwrap()
        });
    }

    let gen = quote! {

        #[derive(::sailfish::TemplateOnce, Default)]
        #ast
    };

    gen.into()
}
