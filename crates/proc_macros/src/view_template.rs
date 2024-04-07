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

    let struct_name = &ast.ident;

    let gen = quote! {
        #[derive(::sailfish::TemplateOnce, Default)]
        #ast

        impl ::web_core::view_template::ViewTemplate for #struct_name {
            fn set_title(&mut self, title: String) -> &mut Self {
                self._base.title = title;

                self
            }

            fn set_description(&mut self, description: String) -> &mut Self {
                self._base.description = description;

                self
            }

            fn set_language(&mut self, language: String) -> &mut Self {
                self._base.language = language;

                self
            }
        }
    };

    gen.into()
}
