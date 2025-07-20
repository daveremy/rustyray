//! Implementation of the #[remote] macro

use crate::utils::{is_object_ref_type, is_result_type, extract_result_ok_type};
use darling::FromMeta;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{parse_macro_input, ItemFn, ReturnType};

#[derive(Debug, FromMeta)]
struct RemoteArgs {
    #[darling(default)]
    num_cpus: Option<f64>,
    #[darling(default)]
    num_gpus: Option<f64>,
}

pub fn remote_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = match darling::ast::NestedMeta::parse_meta_list(args.into()) {
        Ok(v) => v,
        Err(e) => return TokenStream::from(darling::Error::from(e).write_errors()),
    };
    
    let args = match RemoteArgs::from_list(&args) {
        Ok(v) => v,
        Err(e) => return TokenStream::from(e.write_errors()),
    };
    
    let input_fn = parse_macro_input!(input as ItemFn);
    
    // Validate function
    if let Err(e) = validate_remote_function(&input_fn) {
        return TokenStream::from(e.to_compile_error());
    }
    
    // Generate the remote function module
    let result = generate_remote_module(&input_fn, &args);
    
    TokenStream::from(result)
}

fn validate_remote_function(func: &ItemFn) -> syn::Result<()> {
    // Check for self parameter
    if func.sig.inputs.iter().any(|arg| matches!(arg, syn::FnArg::Receiver(_))) {
        return Err(syn::Error::new_spanned(
            &func.sig.ident,
            "#[remote] cannot be used on methods with self parameter",
        ));
    }
    
    // Check for const
    if func.sig.constness.is_some() {
        return Err(syn::Error::new_spanned(
            &func.sig.constness,
            "#[remote] functions cannot be const",
        ));
    }
    
    // Check for unsafe
    if func.sig.unsafety.is_some() {
        return Err(syn::Error::new_spanned(
            &func.sig.unsafety,
            "#[remote] functions cannot be unsafe",
        ));
    }
    
    // Check for lifetime parameters
    if func.sig.generics.lifetimes().count() > 0 {
        return Err(syn::Error::new_spanned(
            &func.sig.generics,
            "#[remote] functions cannot have lifetime parameters",
        ));
    }
    
    // Check for generic parameters
    if !func.sig.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &func.sig.generics,
            "#[remote] functions cannot have generic parameters (yet)",
        ));
    }
    
    Ok(())
}

fn generate_remote_module(func: &ItemFn, args: &RemoteArgs) -> proc_macro2::TokenStream {
    let fn_name = &func.sig.ident;
    let fn_inputs = &func.sig.inputs;
    let fn_output = &func.sig.output;
    let fn_body = &func.block;
    let fn_vis = &func.vis;
    let is_async = func.sig.asyncness.is_some();
    
    let mod_name = Ident::new(&format!("{}_remote", fn_name), Span::call_site());
    
    // Extract parameter names and types
    let params: Vec<_> = fn_inputs
        .iter()
        .filter_map(|arg| match arg {
            syn::FnArg::Typed(pat_type) => Some(pat_type),
            _ => None,
        })
        .collect();
    
    let param_names = params
        .iter()
        .map(|param| match &*param.pat {
            syn::Pat::Ident(ident) => Ok(&ident.ident),
            pat => Err(syn::Error::new_spanned(
                pat,
                "Remote functions must use simple identifiers for parameters",
            )),
        })
        .collect::<syn::Result<Vec<_>>>();
    
    let param_names = match param_names {
        Ok(names) => names,
        Err(e) => return e.to_compile_error(),
    };
    
    let param_types: Vec<_> = params.iter().map(|param| &*param.ty).collect();
    
    // Generate the appropriate arg calls based on parameter types
    let add_arg_calls: Vec<_> = params
        .iter()
        .zip(param_names.iter())
        .map(|(param, name)| {
            if is_object_ref_type(&param.ty) {
                quote! { .arg_ref(&#name) }
            } else {
                quote! { .arg(#name) }
            }
        })
        .collect();
    
    // Extract return type and check if it's already a Result
    let (return_type, object_ref_type, returns_result) = match fn_output {
        ReturnType::Default => (quote! { () }, quote! { () }, false),
        ReturnType::Type(_, ty) => {
            let is_result = is_result_type(ty);
            if is_result {
                // Extract T from Result<T, E>
                let inner_type = extract_result_ok_type(ty).unwrap_or_else(|| quote! { () });
                (quote! { #ty }, inner_type, true)
            } else {
                (quote! { #ty }, quote! { #ty }, false)
            }
        }
    };
    
    // Generate the function execution for registration
    // The task_function macro expects a Result, so we need to wrap non-Result returns
    let function_execution = if returns_result {
        // Function already returns Result, just call it directly
        if is_async {
            quote! {
                |#(#param_names: #param_types),*| async move #fn_body
            }
        } else {
            quote! {
                |#(#param_names: #param_types),*| async move {
                    let sync_fn = move || #fn_body;
                    sync_fn()
                }
            }
        }
    } else {
        // Function doesn't return Result, wrap it
        if is_async {
            quote! {
                |#(#param_names: #param_types),*| async move {
                    let result: #return_type = #fn_body;
                    Ok::<#return_type, rustyray_core::RustyRayError>(result)
                }
            }
        } else {
            quote! {
                |#(#param_names: #param_types),*| async move {
                    let sync_fn = move || #fn_body;
                    let result: #return_type = sync_fn();
                    Ok::<#return_type, rustyray_core::RustyRayError>(result)
                }
            }
        }
    };
    
    // Generate resource requirements
    let resource_config = if args.num_cpus.is_some() || args.num_gpus.is_some() {
        let num_cpus = args.num_cpus.unwrap_or(1.0);
        let num_gpus = args.num_gpus.unwrap_or(0.0);
        quote! {
            .num_cpus(#num_cpus)
            .num_gpus(#num_gpus)
        }
    } else {
        quote! {}
    };
    
    // Generate the module
    quote! {
        // Original function remains accessible
        #func
        
        // Generated module for remote execution
        #fn_vis mod #mod_name {
            use super::*;
            use rustyray_core::{TaskBuilder, ObjectRef, Result, RustyRayError};
            use rustyray_core::runtime;
            
            /// Execute this function remotely
            pub fn remote(#fn_inputs) -> impl std::future::Future<Output = Result<ObjectRef<#object_ref_type>>> {
                async move {
                    let task_system = runtime::global()?.task_system();
                    
                    TaskBuilder::new(stringify!(#fn_name))
                        #(
                            #add_arg_calls
                        )*
                        #resource_config
                        .submit::<#object_ref_type>(&task_system)
                        .await
                }
            }
            
            // Registration with linkme
            #[linkme::distributed_slice(rustyray_core::runtime::REMOTE_FUNCTIONS)]
            static REGISTER: rustyray_core::runtime::RemoteFunctionRegistration = 
                rustyray_core::runtime::RemoteFunctionRegistration {
                    name: stringify!(#fn_name),
                    register: |system| {
                        // The actual function that will be called
                        let func = rustyray_core::task_function!(
                            #function_execution
                        );
                        if let Err(e) = system.register_function(stringify!(#fn_name), func) {
                            eprintln!("Warning: Failed to register remote function '{}': {}", 
                                     stringify!(#fn_name), e);
                        }
                    },
                };
        }
    }
}