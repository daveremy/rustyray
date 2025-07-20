//! Implementation of the #[actor] and #[actor_methods] macros

use darling::FromMeta;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
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
    
    // Extract struct info
    let struct_name = &input_struct.ident;
    let struct_vis = &input_struct.vis;
    let struct_generics = &input_struct.generics;
    
    // Validate no generics for now
    if !struct_generics.params.is_empty() {
        return TokenStream::from(
            syn::Error::new_spanned(
                struct_generics,
                "#[actor] does not support generic parameters yet"
            ).to_compile_error()
        );
    }
    
    // Generate the handle struct name
    let handle_name = Ident::new(&format!("{}Handle", struct_name), Span::call_site());
    let actor_id_name = Ident::new(&format!("{}_ACTOR_ID", struct_name.to_string().to_uppercase()), Span::call_site());
    
    // Generate message enum name - these will be used by actor_methods
    let _message_enum_name = Ident::new(&format!("{}Message", struct_name), Span::call_site());
    let _response_enum_name = Ident::new(&format!("{}Response", struct_name), Span::call_site());
    
    // Store actor metadata in a const that can be used by actor_methods
    let metadata = quote! {
        // Store metadata for use by #[actor_methods]
        #[doc(hidden)]
        const #actor_id_name: &str = stringify!(#struct_name);
    };
    
    // Generate the output
    let output = quote! {
        // Original struct with added derives
        #[derive(serde::Serialize, serde::Deserialize)]
        #input_struct
        
        #metadata
        
        // Actor handle for typed remote access
        #[derive(Clone, Debug)]
        #struct_vis struct #handle_name {
            actor_ref: rustyray_core::actor::ActorRef,
            _phantom: std::marker::PhantomData<#struct_name>,
        }
        
        impl #handle_name {
            #[doc(hidden)]
            pub fn new(actor_ref: rustyray_core::actor::ActorRef) -> Self {
                Self {
                    actor_ref,
                    _phantom: std::marker::PhantomData,
                }
            }
        }
    };
    
    TokenStream::from(output)
}

pub fn actor_methods_impl(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_impl = parse_macro_input!(input as ItemImpl);
    
    // Ensure no trait is being implemented
    if input_impl.trait_.is_some() {
        return TokenStream::from(
            syn::Error::new_spanned(
                &input_impl,
                "#[actor_methods] must be used on inherent impl blocks, not trait implementations"
            ).to_compile_error()
        );
    }
    
    // Extract the type we're implementing for
    let self_ty = match &*input_impl.self_ty {
        syn::Type::Path(type_path) => &type_path.path,
        _ => {
            return TokenStream::from(
                syn::Error::new_spanned(
                    &input_impl.self_ty,
                    "#[actor_methods] can only be used on simple type paths"
                ).to_compile_error()
            );
        }
    };
    
    let struct_name = match self_ty.segments.last() {
        Some(segment) => &segment.ident,
        None => {
            return TokenStream::from(
                syn::Error::new_spanned(
                    self_ty,
                    "Could not determine struct name"
                ).to_compile_error()
            );
        }
    };
    
    // Generate associated type names
    let handle_name = Ident::new(&format!("{}Handle", struct_name), Span::call_site());
    let message_enum_name = Ident::new(&format!("{}Message", struct_name), Span::call_site());
    let response_enum_name = Ident::new(&format!("{}Response", struct_name), Span::call_site());
    let wrapper_name = Ident::new(&format!("{}ActorWrapper", struct_name), Span::call_site());
    
    // Collect methods and categorize them
    let mut generated_constructors = Vec::new();
    let mut message_variants = Vec::new();
    let mut response_variants = Vec::new();
    let mut handle_methods = Vec::new();
    let mut message_handlers = Vec::new();
    
    for item in &input_impl.items {
        if let syn::ImplItem::Fn(method) = item {
            let method_name = &method.sig.ident;
            let method_vis = &method.vis;
            
            // Check if this is a constructor (returns Self)
            let is_constructor = match &method.sig.output {
                syn::ReturnType::Default => false,
                syn::ReturnType::Type(_, ty) => {
                    matches!(&**ty, syn::Type::Path(type_path) 
                        if type_path.path.is_ident("Self"))
                }
            };
            
            if is_constructor {
                // Validate constructor doesn't have self parameter
                if method.sig.receiver().is_some() {
                    return TokenStream::from(
                        syn::Error::new_spanned(
                            &method.sig,
                            "Constructor methods cannot have self parameter"
                        ).to_compile_error()
                    );
                }
                
                // Generate remote constructor in impl block
                let params = &method.sig.inputs;
                let param_names: Vec<_> = params
                    .iter()
                    .filter_map(|arg| match arg {
                        syn::FnArg::Typed(pat_type) => {
                            match &*pat_type.pat {
                                syn::Pat::Ident(ident) => Some(&ident.ident),
                                _ => None,
                            }
                        }
                        _ => None,
                    })
                    .collect();
                
                // Constructor creates actor remotely - use constructor name as remote method name
                let remote_method_name = if method_name == "new" {
                    Ident::new("remote", Span::call_site())
                } else {
                    Ident::new(&format!("remote_{}", method_name), Span::call_site())
                };
                
                let constructor_method = quote! {
                    #method_vis async fn #remote_method_name(#params) -> rustyray_core::Result<#handle_name> {
                        let runtime = rustyray_core::runtime::global()?;
                        let actor_system = runtime.actor_system();
                        
                        let actor = Self::#method_name(#(#param_names),*);
                        let wrapper = #wrapper_name(actor);
                        let actor_ref = actor_system.create_actor(wrapper).await?;
                        
                        Ok(#handle_name::new(actor_ref))
                    }
                };
                
                generated_constructors.push(constructor_method);
            } else {
                // Regular method - must have self receiver
                if method.sig.receiver().is_none() {
                    return TokenStream::from(
                        syn::Error::new_spanned(
                            &method.sig,
                            "Actor methods must have &self or &mut self parameter. Use constructor pattern (returns Self) for static methods."
                        ).to_compile_error()
                    );
                }
                
                // Extract method parameters (skip self)
                let params: Vec<_> = method.sig.inputs
                    .iter()
                    .filter_map(|arg| match arg {
                        syn::FnArg::Typed(pat_type) => Some(pat_type),
                        _ => None,
                    })
                    .collect();
                
                let param_types: Vec<_> = params.iter().map(|p| &*p.ty).collect();
                let param_names: Vec<_> = params
                    .iter()
                    .filter_map(|p| match &*p.pat {
                        syn::Pat::Ident(ident) => Some(&ident.ident),
                        _ => None,
                    })
                    .collect();
                
                // Create message variant
                let variant_name = Ident::new(
                    &format!("{}{}", 
                        method_name.to_string().chars().next().unwrap().to_uppercase(),
                        &method_name.to_string()[1..]
                    ),
                    Span::call_site()
                );
                
                if param_types.is_empty() {
                    message_variants.push(quote! {
                        #variant_name
                    });
                } else {
                    message_variants.push(quote! {
                        #variant_name { #(#param_names: #param_types),* }
                    });
                }
                
                // Create response variant based on method return type
                let return_type = match &method.sig.output {
                    syn::ReturnType::Default => quote! { () },
                    syn::ReturnType::Type(_, ty) => quote! { #ty },
                };
                
                response_variants.push(quote! {
                    #variant_name(#return_type)
                });
                
                // Create handle method
                let handle_method = if param_types.is_empty() {
                    quote! {
                        #method_vis async fn #method_name(&self) -> rustyray_core::Result<rustyray_core::ObjectRef<#return_type>> {
                            let msg = #message_enum_name::#variant_name;
                            let response = self.actor_ref.call(Box::new(msg)).await?;
                            
                            match response.downcast::<#response_enum_name>() {
                                Ok(resp) => match *resp {
                                    #response_enum_name::#variant_name(value) => {
                                        // Put the value into the object store
                                        let runtime = rustyray_core::runtime::global()?;
                                        let task_system = runtime.task_system();
                                        task_system.put(value).await
                                    }
                                    _ => Err(rustyray_core::RustyRayError::Internal(
                                        format!("Unexpected response type for method '{}'. Expected variant '{}' but got different variant", 
                                               stringify!(#method_name), stringify!(#variant_name))
                                    )),
                                },
                                Err(_) => Err(rustyray_core::RustyRayError::Internal(
                                    format!("Failed to downcast response for method '{}'. Expected response type '{}' but got incompatible type", 
                                           stringify!(#method_name), stringify!(#response_enum_name))
                                )),
                            }
                        }
                    }
                } else {
                    quote! {
                        #method_vis async fn #method_name(&self, #(#param_names: #param_types),*) -> rustyray_core::Result<rustyray_core::ObjectRef<#return_type>> {
                            let msg = #message_enum_name::#variant_name { #(#param_names),* };
                            let response = self.actor_ref.call(Box::new(msg)).await?;
                            
                            match response.downcast::<#response_enum_name>() {
                                Ok(resp) => match *resp {
                                    #response_enum_name::#variant_name(value) => {
                                        // Put the value into the object store
                                        let runtime = rustyray_core::runtime::global()?;
                                        let task_system = runtime.task_system();
                                        task_system.put(value).await
                                    }
                                    _ => Err(rustyray_core::RustyRayError::Internal(
                                        format!("Unexpected response type for method '{}'. Expected variant '{}' but got different variant", 
                                               stringify!(#method_name), stringify!(#variant_name))
                                    )),
                                },
                                Err(_) => Err(rustyray_core::RustyRayError::Internal(
                                    format!("Failed to downcast response for method '{}'. Expected response type '{}' but got incompatible type", 
                                           stringify!(#method_name), stringify!(#response_enum_name))
                                )),
                            }
                        }
                    }
                };
                
                handle_methods.push(handle_method);
                
                // Create message handler
                let is_async = method.sig.asyncness.is_some();
                let handler = if param_types.is_empty() {
                    if is_async {
                        quote! {
                            #message_enum_name::#variant_name => {
                                let result = self.0.#method_name().await;
                                Ok(Box::new(#response_enum_name::#variant_name(result)))
                            }
                        }
                    } else {
                        quote! {
                            #message_enum_name::#variant_name => {
                                let result = self.0.#method_name();
                                Ok(Box::new(#response_enum_name::#variant_name(result)))
                            }
                        }
                    }
                } else {
                    if is_async {
                        quote! {
                            #message_enum_name::#variant_name { #(#param_names),* } => {
                                let result = self.0.#method_name(#(#param_names),*).await;
                                Ok(Box::new(#response_enum_name::#variant_name(result)))
                            }
                        }
                    } else {
                        quote! {
                            #message_enum_name::#variant_name { #(#param_names),* } => {
                                let result = self.0.#method_name(#(#param_names),*);
                                Ok(Box::new(#response_enum_name::#variant_name(result)))
                            }
                        }
                    }
                };
                
                message_handlers.push(handler);
            }
        }
    }
    
    // Generate the complete implementation
    let output = quote! {
        // Original impl block
        #input_impl
        
        // Constructor methods in the struct impl
        impl #struct_name {
            #(#generated_constructors)*
        }
        
        // Message enum
        #[doc(hidden)]
        #[derive(serde::Serialize, serde::Deserialize)]
        enum #message_enum_name {
            #(#message_variants,)*
        }
        
        // Response enum
        #[doc(hidden)]
        #[derive(serde::Serialize, serde::Deserialize)]
        enum #response_enum_name {
            #(#response_variants,)*
        }
        
        // Actor wrapper that implements the Actor trait
        #[doc(hidden)]
        struct #wrapper_name(#struct_name);
        
        #[async_trait::async_trait]
        impl rustyray_core::actor::Actor for #wrapper_name {
            async fn handle(&mut self, msg: Box<dyn std::any::Any + Send>) -> rustyray_core::Result<Box<dyn std::any::Any + Send>> {
                match msg.downcast::<#message_enum_name>() {
                    Ok(msg) => {
                        match *msg {
                            #(#message_handlers)*
                        }
                    }
                    Err(_) => Err(rustyray_core::RustyRayError::Internal(
                        format!("Unknown message type for actor '{}'. Message could not be downcast to '{}'", 
                               stringify!(#struct_name), stringify!(#message_enum_name))
                    )),
                }
            }
        }
        
        // Handle methods implementation
        impl #handle_name {
            #(#handle_methods)*
        }
    };
    
    TokenStream::from(output)
}