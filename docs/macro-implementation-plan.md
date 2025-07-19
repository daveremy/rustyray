# RustyRay Macro Implementation Plan

## Overview

This document provides a detailed implementation plan for the RustyRay macro system that will dramatically improve API ergonomics. The plan is structured to allow incremental development with testable milestones.

## Project Structure

```
rustyray/
├── rustyray-macros/          # New proc-macro crate
│   ├── src/
│   │   ├── lib.rs           # Macro exports
│   │   ├── remote.rs        # #[remote] implementation
│   │   ├── actor.rs         # #[actor] implementation
│   │   ├── actor_methods.rs # #[actor_methods] implementation
│   │   ├── main.rs          # #[rustyray::main] implementation
│   │   └── utils.rs         # Shared utilities
│   └── Cargo.toml
├── rustyray-core/           # Renamed from src/
│   └── ... (existing code)
├── src/                     # New facade crate
│   ├── lib.rs              # Re-exports and prelude
│   └── runtime.rs          # Global runtime management
└── examples/
    └── macro_examples/      # New macro-based examples
```

## Phase 1: Basic Infrastructure (Week 1)

### 1.1 Create Macro Crate

```toml
# rustyray-macros/Cargo.toml
[package]
name = "rustyray-macros"
version = "0.1.0"
edition = "2021"

[lib]
proc-macro = true

[dependencies]
syn = { version = "2.0", features = ["full", "extra-traits"] }
quote = "1.0"
proc-macro2 = "1.0"
```

### 1.2 Basic Macro Structure

```rust
// rustyray-macros/src/lib.rs
use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn remote(attr: TokenStream, item: TokenStream) -> TokenStream {
    remote::remote_impl(attr, item)
}

#[proc_macro_attribute]
pub fn actor(attr: TokenStream, item: TokenStream) -> TokenStream {
    actor::actor_impl(attr, item)
}

#[proc_macro_attribute]
pub fn actor_methods(attr: TokenStream, item: TokenStream) -> TokenStream {
    actor_methods::actor_methods_impl(attr, item)
}

#[proc_macro_attribute]
pub fn main(attr: TokenStream, item: TokenStream) -> TokenStream {
    main::main_impl(attr, item)
}
```

## Phase 2: Remote Function Macro (Week 2-3)

### 2.1 Function Parsing

```rust
// rustyray-macros/src/remote.rs
use syn::{parse_macro_input, ItemFn, ReturnType, FnArg, Pat};
use quote::quote;
use proc_macro2::{TokenStream, Ident, Span};

struct RemoteFunction {
    original_fn: ItemFn,
    fn_name: Ident,
    args: Vec<(Ident, Box<syn::Type>)>,
    return_type: Box<syn::Type>,
    is_async: bool,
    resources: Resources,
}

struct Resources {
    num_cpus: f64,
    num_gpus: f64,
}

impl RemoteFunction {
    fn parse(attr: TokenStream, item: TokenStream) -> syn::Result<Self> {
        let original_fn = parse_macro_input!(item as ItemFn);
        
        // Parse attributes (num_cpus = 2.0, etc.)
        let resources = parse_resources(attr)?;
        
        // Extract function info
        let fn_name = original_fn.sig.ident.clone();
        let is_async = original_fn.sig.asyncness.is_some();
        
        // Parse arguments
        let args = parse_arguments(&original_fn.sig)?;
        
        // Parse return type
        let return_type = parse_return_type(&original_fn.sig)?;
        
        Ok(RemoteFunction {
            original_fn,
            fn_name,
            args,
            return_type,
            is_async,
            resources,
        })
    }
}
```

### 2.2 Code Generation

```rust
impl RemoteFunction {
    fn generate(&self) -> TokenStream {
        let original_fn = &self.original_fn;
        let fn_name = &self.fn_name;
        let mod_name = &self.fn_name;
        let fn_name_str = fn_name.to_string();
        
        // Generate argument serialization
        let arg_names: Vec<_> = self.args.iter().map(|(name, _)| name).collect();
        let arg_types: Vec<_> = self.args.iter().map(|(_, ty)| ty).collect();
        
        // Generate the remote call module
        let remote_mod = self.generate_remote_module();
        
        // Generate registration
        let registration = self.generate_registration();
        
        quote! {
            #original_fn
            
            #remote_mod
            
            #registration
        }
    }
    
    fn generate_remote_module(&self) -> TokenStream {
        let mod_name = &self.fn_name;
        let fn_name = &self.fn_name;
        let fn_name_str = fn_name.to_string();
        let return_type = &self.return_type;
        let arg_names: Vec<_> = self.args.iter().map(|(name, _)| name).collect();
        let arg_types: Vec<_> = self.args.iter().map(|(_, ty)| ty).collect();
        
        quote! {
            pub mod #mod_name {
                use super::*;
                use rustyray::task::{ObjectRef, TaskBuilder};
                use rustyray::error::Result;
                
                pub async fn remote(#(#arg_names: #arg_types),*) -> Result<ObjectRef<#return_type>> {
                    let system = rustyray::runtime::global_system()?;
                    TaskBuilder::new(#fn_name_str)
                        #(.arg(#arg_names))*
                        .submit(&system)
                        .await
                }
                
                pub async fn remote_on(
                    system: &rustyray::task::TaskSystem,
                    #(#arg_names: #arg_types),*
                ) -> Result<ObjectRef<#return_type>> {
                    TaskBuilder::new(#fn_name_str)
                        #(.arg(#arg_names))*
                        .submit(system)
                        .await
                }
            }
        }
    }
    
    fn generate_registration(&self) -> TokenStream {
        let fn_name = &self.fn_name;
        let fn_name_str = fn_name.to_string();
        let arg_types: Vec<_> = self.args.iter().map(|(_, ty)| ty).collect();
        let return_type = &self.return_type;
        
        // Use linkme for compile-time registration
        quote! {
            #[linkme::distributed_slice(rustyray::REMOTE_FUNCTIONS)]
            #[linkme(crate = rustyray::linkme)]
            static #fn_name: rustyray::RemoteFunctionRegistration = 
                rustyray::RemoteFunctionRegistration {
                    name: #fn_name_str,
                    register: |registry| {
                        rustyray::register_task_function!(
                            #fn_name_str,
                            |#(#arg_names: #arg_types),*| async move {
                                Ok::<#return_type, rustyray::error::RustyRayError>(
                                    #fn_name(#(#arg_names),*).await
                                )
                            }
                        )
                    },
                };
        }
    }
}
```

## Phase 3: Actor Macros (Week 4-5)

### 3.1 Actor Struct Macro

```rust
// rustyray-macros/src/actor.rs
use syn::{parse_macro_input, ItemStruct, Fields};

struct ActorStruct {
    original: ItemStruct,
    name: Ident,
    fields: Vec<(Ident, Box<syn::Type>)>,
}

impl ActorStruct {
    fn generate(&self) -> TokenStream {
        let original = &self.original;
        let actor_name = &self.name;
        let handle_name = Ident::new(&format!("{}Handle", actor_name), Span::call_site());
        
        quote! {
            #[derive(serde::Serialize, serde::Deserialize)]
            #original
            
            // Generate the handle type
            #[derive(Clone)]
            pub struct #handle_name {
                inner: rustyray::actor::ActorRef,
            }
        }
    }
}
```

### 3.2 Actor Methods Macro

```rust
// rustyray-macros/src/actor_methods.rs
struct ActorImpl {
    actor_name: Ident,
    methods: Vec<Method>,
}

struct Method {
    name: Ident,
    is_async: bool,
    is_mut: bool,
    args: Vec<(Ident, Box<syn::Type>)>,
    return_type: Box<syn::Type>,
}

impl ActorImpl {
    fn generate(&self) -> TokenStream {
        let actor_name = &self.actor_name;
        let handle_name = Ident::new(&format!("{}Handle", actor_name), Span::call_site());
        let message_enum = self.generate_message_enum();
        let response_enum = self.generate_response_enum();
        let actor_impl = self.generate_actor_impl();
        let handle_impl = self.generate_handle_impl();
        let constructor = self.generate_constructor();
        
        quote! {
            #message_enum
            #response_enum
            #actor_impl
            #handle_impl
            #constructor
        }
    }
    
    fn generate_message_enum(&self) -> TokenStream {
        let actor_name = &self.actor_name;
        let enum_name = Ident::new(&format!("{}Message", actor_name), Span::call_site());
        
        let variants: Vec<_> = self.methods.iter().map(|method| {
            let name = &method.name;
            let variant_name = to_pascal_case(&name.to_string());
            let args = &method.args;
            
            if args.is_empty() {
                quote! { #variant_name }
            } else {
                let types: Vec<_> = args.iter().map(|(_, ty)| ty).collect();
                quote! { #variant_name(#(#types),*) }
            }
        }).collect();
        
        quote! {
            #[derive(Debug)]
            enum #enum_name {
                #(#variants),*
            }
        }
    }
    
    fn generate_handle_impl(&self) -> TokenStream {
        let actor_name = &self.actor_name;
        let handle_name = Ident::new(&format!("{}Handle", actor_name), Span::call_site());
        
        let methods: Vec<_> = self.methods.iter().map(|method| {
            let method_name = &method.name;
            let args = &method.args;
            let return_type = &method.return_type;
            
            let arg_names: Vec<_> = args.iter().map(|(name, _)| name).collect();
            let arg_types: Vec<_> = args.iter().map(|(_, ty)| ty).collect();
            
            quote! {
                pub async fn #method_name(&self, #(#arg_names: #arg_types),*) 
                    -> rustyray::error::Result<rustyray::task::ObjectRef<#return_type>> 
                {
                    // Implementation that sends message and returns ObjectRef
                    todo!()
                }
            }
        }).collect();
        
        quote! {
            impl #handle_name {
                #(#methods)*
            }
        }
    }
}
```

## Phase 4: Runtime Management (Week 5)

### 4.1 Global Runtime

```rust
// src/runtime.rs
use once_cell::sync::OnceCell;
use std::sync::Arc;
use crate::error::{Result, RustyRayError};
use crate::actor::ActorSystem;
use crate::task::TaskSystem;

static RUNTIME: OnceCell<Runtime> = OnceCell::new();

pub struct Runtime {
    actor_system: Arc<ActorSystem>,
    task_system: Arc<TaskSystem>,
}

impl Runtime {
    pub fn init() -> Result<()> {
        let actor_system = Arc::new(ActorSystem::new());
        let task_system = Arc::new(TaskSystem::new(actor_system.clone()));
        
        let runtime = Runtime {
            actor_system,
            task_system,
        };
        
        RUNTIME.set(runtime)
            .map_err(|_| RustyRayError::Internal("Runtime already initialized".to_string()))
    }
    
    pub fn global() -> Result<&'static Runtime> {
        RUNTIME.get()
            .ok_or_else(|| RustyRayError::Internal("Runtime not initialized".to_string()))
    }
    
    pub fn shutdown() -> Result<()> {
        // Implementation
        todo!()
    }
}

pub fn global_system() -> Result<&'static TaskSystem> {
    Ok(&Runtime::global()?.task_system)
}
```

### 4.2 Main Macro

```rust
// rustyray-macros/src/main.rs
pub fn main_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let main_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &main_fn.sig.ident;
    let fn_body = &main_fn.block;
    
    quote! {
        fn #fn_name() -> rustyray::error::Result<()> {
            rustyray::runtime::Runtime::init()?;
            
            let runtime = tokio::runtime::Runtime::new()?;
            let result = runtime.block_on(async #fn_body);
            
            rustyray::runtime::Runtime::shutdown()?;
            result
        }
    }
}
```

## Phase 5: Testing Infrastructure (Week 6)

### 5.1 Macro Testing

```rust
// rustyray-macros/tests/remote.rs
use rustyray_macros::remote;

#[test]
fn test_remote_function_generation() {
    // Use trybuild for compile tests
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/remote_pass.rs");
    t.compile_fail("tests/ui/remote_fail.rs");
}

// tests/ui/remote_pass.rs
#[rustyray::remote]
async fn add(x: i32, y: i32) -> i32 {
    x + y
}

#[rustyray::remote(num_cpus = 2.0)]
async fn heavy_compute(data: Vec<f64>) -> f64 {
    data.iter().sum()
}
```

### 5.2 Integration Tests

```rust
// tests/macro_integration.rs
#[rustyray::main]
async fn test_basic_remote() -> rustyray::Result<()> {
    #[rustyray::remote]
    async fn double(x: i32) -> i32 {
        x * 2
    }
    
    let result = double::remote(21).await?.get().await?;
    assert_eq!(result, 42);
    Ok(())
}

#[test]
fn test_actor_macro() {
    // Test actor generation
}
```

## Migration Strategy

### Step 1: Parallel Development
- Keep existing API working
- Develop macros in parallel
- Test with new examples

### Step 2: Gradual Migration
- Update internal code to use macros
- Provide migration guide
- Deprecate old APIs

### Step 3: Full Transition
- Remove deprecated APIs
- Update all documentation
- Release 1.0

## Performance Optimizations

1. **Compile-time Registration**: Use `linkme` to eliminate runtime overhead
2. **Zero-cost Abstractions**: Macros generate direct calls, no indirection
3. **Type-safe Serialization**: Generate specialized serialization code
4. **Actor Message Batching**: Generated code can batch messages

## Documentation Plan

1. **Macro Documentation**: Use doc comments in macro output
2. **Migration Guide**: Step-by-step guide from old to new API
3. **Examples**: Comprehensive examples for each macro
4. **Best Practices**: Guidelines for macro usage

## Success Metrics

1. **API Simplicity**: 50% reduction in boilerplate code
2. **Type Safety**: Compile-time verification of all remote calls
3. **Performance**: No runtime overhead vs manual implementation
4. **Developer Experience**: Positive feedback from early adopters

## Risks and Mitigations

1. **Macro Complexity**: Mitigate with thorough testing and documentation
2. **Compilation Time**: Monitor and optimize macro expansion
3. **Debugging**: Provide good error messages and expansion tools
4. **Breaking Changes**: Use feature flags for gradual adoption

## Timeline Summary

- **Week 1**: Basic infrastructure and project setup
- **Week 2-3**: Remote function macro implementation
- **Week 4-5**: Actor macros and runtime management
- **Week 6**: Testing, documentation, and polish

This plan provides a clear path to implementing the macro system that will transform RustyRay's API from verbose to elegant, matching Ray's simplicity while maintaining Rust's safety guarantees.