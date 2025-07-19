//! Shared utilities for macro implementations

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

/// Implementation of the #[rustyray::main] macro
pub fn main_impl(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);
    
    let fn_name = &input_fn.sig.ident;
    let fn_body = &input_fn.block;
    let fn_output = &input_fn.sig.output;
    
    let result = quote! {
        #[tokio::main]
        async fn #fn_name() #fn_output {
            // Initialize the runtime
            rustyray_core::runtime::init().expect("Failed to initialize RustyRay runtime");
            
            // Run the user's function
            let result = async move #fn_body.await;
            
            // Shutdown cleanly
            rustyray_core::runtime::shutdown().expect("Failed to shutdown RustyRay runtime");
            
            result
        }
    };
    
    TokenStream::from(result)
}

/// Helper to create good error messages with proper spans
#[allow(dead_code)]
pub fn error_with_span(span: proc_macro2::Span, message: &str) -> syn::Error {
    syn::Error::new(span, message)
}