//! Implementation of the #[actor] and #[actor_methods] macros

use darling::FromMeta;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemStruct, ItemImpl};

#[derive(Debug, FromMeta)]
#[allow(dead_code)]
struct ActorArgs {
    #[darling(default)]
    num_cpus: Option<f64>,
    #[darling(default)]
    num_gpus: Option<f64>,
}

pub fn actor_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = match darling::ast::NestedMeta::parse_meta_list(args.into()) {
        Ok(v) => v,
        Err(e) => return TokenStream::from(darling::Error::from(e).write_errors()),
    };
    
    let _args = match ActorArgs::from_list(&args) {
        Ok(v) => v,
        Err(e) => return TokenStream::from(e.write_errors()),
    };
    
    let input_struct = parse_macro_input!(input as ItemStruct);
    
    // TODO: Implement actor macro
    // For now, just return the original struct
    TokenStream::from(quote! { #input_struct })
}

pub fn actor_methods_impl(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_impl = parse_macro_input!(input as ItemImpl);
    
    // TODO: Implement actor_methods macro
    // For now, just return the original impl
    TokenStream::from(quote! { #input_impl })
}