//! Shared utilities for macro implementations

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

/// Check if a type is ObjectRef, handling various patterns:
/// - Simple: ObjectRef<T>
/// - Qualified: rustyray::ObjectRef<T>, rustyray_core::ObjectRef<T>
/// - References: &ObjectRef<T>
/// - Type aliases are NOT handled (would require semantic analysis)
pub fn is_object_ref_type(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Path(type_path) => {
            // Check all possible ObjectRef patterns
            let path = &type_path.path;

            // Handle simple case: ObjectRef<T>
            if path.segments.len() == 1 {
                return path.segments[0].ident == "ObjectRef";
            }

            // Handle qualified paths
            if path.segments.len() >= 2 {
                let last = &path.segments[path.segments.len() - 1];
                if last.ident == "ObjectRef" {
                    // Check for common qualifiers
                    let prefix_segments: Vec<_> = path
                        .segments
                        .iter()
                        .take(path.segments.len() - 1)
                        .map(|s| s.ident.to_string())
                        .collect();

                    // Check for rustyray::ObjectRef or rustyray_core::ObjectRef
                    let segments: Vec<&str> = prefix_segments.iter().map(|s| s.as_str()).collect();
                    match segments.as_slice() {
                        ["rustyray"] => return true,
                        ["rustyray_core"] => return true,
                        ["crate"] => return true,
                        ["super"] => return true,
                        _ => {
                            // Also check if it ends with a module that re-exports ObjectRef
                            if prefix_segments
                                .last()
                                .map(|s| s == "rustyray" || s == "rustyray_core")
                                .unwrap_or(false)
                            {
                                return true;
                            }
                        }
                    }
                }
            }

            false
        }
        syn::Type::Reference(type_ref) => is_object_ref_type(&type_ref.elem),
        _ => false,
    }
}

/// Check if a type is Result<T, E>
pub fn is_result_type(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Path(type_path) => {
            let path = &type_path.path;

            // Check the last segment
            if let Some(segment) = path.segments.last() {
                if segment.ident == "Result" {
                    // Verify it has angle bracketed args (Result<T, E>)
                    matches!(&segment.arguments, syn::PathArguments::AngleBracketed(_))
                } else {
                    false
                }
            } else {
                false
            }
        }
        _ => false,
    }
}

/// Extract the Ok type from Result<T, E>
pub fn extract_result_ok_type(ty: &syn::Type) -> Option<proc_macro2::TokenStream> {
    match ty {
        syn::Type::Path(type_path) => {
            let segment = type_path.path.segments.last()?;

            if segment.ident != "Result" {
                return None;
            }

            if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                if let Some(syn::GenericArgument::Type(ok_type)) = args.args.first() {
                    return Some(quote! { #ok_type });
                }
            }

            None
        }
        _ => None,
    }
}

/// Implementation of the #[rustyray::main] macro
pub fn main_impl(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);

    let fn_name = &input_fn.sig.ident;
    let fn_body = &input_fn.block;
    let fn_output = &input_fn.sig.output;

    // Check if the function returns a Result
    let returns_result = match &fn_output {
        syn::ReturnType::Default => false,
        syn::ReturnType::Type(_, ty) => is_result_type(ty),
    };

    let result = if returns_result {
        // Function already returns Result, propagate errors
        quote! {
            #[tokio::main]
            async fn #fn_name() #fn_output {
                // Initialize the runtime
                rustyray_core::runtime::init()?;

                // Run the user's function
                let result = async move #fn_body.await;

                // Shutdown cleanly
                rustyray_core::runtime::shutdown()?;

                result
            }
        }
    } else {
        // Function doesn't return Result, use expect
        quote! {
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
        }
    };

    TokenStream::from(result)
}

/// Helper to create good error messages with proper spans
#[allow(dead_code)]
pub fn error_with_span(span: proc_macro2::Span, message: &str) -> syn::Error {
    syn::Error::new(span, message)
}
