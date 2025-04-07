use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemStruct};

// Атрибутный макрос #[tgr::obj]
#[proc_macro_attribute]
pub fn module(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);
    let name = &input.ident;

    let expanded = quote! {
        #input

        inventory::submit! {
            &#name as &dyn ::tgr::engine::object::Module
        }
    };

    TokenStream::from(expanded)
}
