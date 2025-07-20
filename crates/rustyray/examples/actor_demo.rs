//! Example demonstrating the #[actor] and #[actor_methods] macros

use rustyray::prelude::*;

#[rustyray::actor]
pub struct Counter {
    value: i32,
    name: String,
}

#[rustyray::actor_methods]
impl Counter {
    // Constructor
    pub fn new(name: String, initial: i32) -> Self {
        Counter { value: initial, name }
    }
    
    // Async method
    pub async fn increment(&mut self) -> i32 {
        self.value += 1;
        println!("{}: Incremented to {}", self.name, self.value);
        self.value
    }
    
    // Sync method
    pub fn get(&self) -> i32 {
        self.value
    }
    
    // Method with parameters
    pub async fn add(&mut self, amount: i32) -> i32 {
        self.value += amount;
        println!("{}: Added {}, new value: {}", self.name, amount, self.value);
        self.value
    }
}

#[rustyray::main]
async fn main() -> Result<()> {
    println!("Actor Demo - Testing actor macros");
    
    // Create a remote actor
    let counter = Counter::remote("TestCounter".to_string(), 0).await?;
    println!("Created remote counter actor");
    
    // Call methods
    let val1 = counter.increment().await?.get().await?;
    println!("After increment: {}", val1);
    
    let val2 = counter.add(10).await?.get().await?;
    println!("After adding 10: {}", val2);
    
    let current = counter.get().await?.get().await?;
    println!("Current value: {}", current);
    
    Ok(())
}