use proc_macro::TokenStream;

mod view_template;

#[proc_macro_attribute]
pub fn web_view_template(args: TokenStream, input: TokenStream) -> TokenStream {
    view_template::impl_attr_web_view_template(args, syn::parse(input).unwrap())
}
