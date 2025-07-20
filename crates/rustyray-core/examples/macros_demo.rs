//! Example demonstrating the new macro API

use rustyray_core::prelude::*;

#[rustyray_macros::remote]
async fn add(x: i32, y: i32) -> i32 {
    x + y
}

#[rustyray_macros::remote(num_cpus = 2.0)]
fn multiply(x: i32, y: i32) -> i32 {
    x * y
}

#[rustyray_macros::main]
async fn main() -> Result<()> {
    // Test async remote function
    let result = add_remote::remote(5, 3).await?;
    println!("Remote add result: {:?}", result.get().await?);

    // Test sync remote function with resources
    let result = multiply_remote::remote(4, 7).await?;
    println!("Remote multiply result: {:?}", result.get().await?);

    Ok(())
}
