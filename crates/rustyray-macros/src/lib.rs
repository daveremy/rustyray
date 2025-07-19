//! RustyRay procedural macros for simplified API
//!
//! This crate provides the `#[remote]`, `#[actor]`, and `#[actor_methods]` macros
//! that dramatically simplify the RustyRay API.

use proc_macro::TokenStream;

mod remote;
mod actor;
mod utils;

/// Mark a function as remotely executable.
///
/// # Examples
///
/// ```ignore
/// #[rustyray::remote]
/// async fn add(x: i32, y: i32) -> i32 {
///     x + y
/// }
///
/// // Usage:
/// let result = add::remote(5, 3).await?;
/// ```
#[proc_macro_attribute]
pub fn remote(args: TokenStream, input: TokenStream) -> TokenStream {
    remote::remote_impl(args, input)
}

/// Mark a struct as an actor.
///
/// # Examples
///
/// ```ignore
/// #[rustyray::actor]
/// pub struct Counter {
///     value: i32,
/// }
/// ```
#[proc_macro_attribute]
pub fn actor(args: TokenStream, input: TokenStream) -> TokenStream {
    actor::actor_impl(args, input)
}

/// Define methods for an actor.
///
/// # Examples
///
/// ```ignore
/// #[rustyray::actor_methods]
/// impl Counter {
///     pub fn new(initial: i32) -> Self {
///         Counter { value: initial }
///     }
///     
///     pub async fn increment(&mut self) -> i32 {
///         self.value += 1;
///         self.value
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn actor_methods(args: TokenStream, input: TokenStream) -> TokenStream {
    actor::actor_methods_impl(args, input)
}

/// Create a RustyRay main function with automatic runtime initialization.
///
/// # Examples
///
/// ```ignore
/// #[rustyray::main]
/// async fn main() -> Result<()> {
///     // Runtime is automatically initialized
///     let result = add::remote(5, 3).await?;
///     println!("Result: {}", result);
///     Ok(())
/// }
/// ```
#[proc_macro_attribute]
pub fn main(args: TokenStream, input: TokenStream) -> TokenStream {
    utils::main_impl(args, input)
}