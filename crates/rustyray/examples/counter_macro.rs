//! Example demonstrating a simple counter actor using the macro API.
//!
//! This shows how much simpler the macro API is compared to the manual
//! implementation with Box<dyn Any> and message enums.

use rustyray::prelude::*;

/// A simple counter actor with typed methods
#[rustyray::actor]
struct Counter {
    count: i32,
    name: String,
}

#[rustyray::actor_methods]
impl Counter {
    /// Create a new counter with a name and initial value
    pub fn new(name: String, initial: i32) -> Self {
        Counter { 
            count: initial,
            name,
        }
    }
    
    /// Create a counter starting at zero
    pub fn zero(name: String) -> Self {
        Counter { count: 0, name }
    }
    
    /// Increment the counter and return the new value
    pub async fn increment(&mut self) -> i32 {
        self.count += 1;
        println!("{}: Incremented to {}", self.name, self.count);
        self.count
    }
    
    /// Decrement the counter and return the new value
    pub async fn decrement(&mut self) -> i32 {
        self.count -= 1;
        println!("{}: Decremented to {}", self.name, self.count);
        self.count
    }
    
    /// Get the current count without modifying it
    pub fn get(&self) -> i32 {
        println!("{}: Current count is {}", self.name, self.count);
        self.count
    }
    
    /// Add a value to the counter
    pub async fn add(&mut self, value: i32) -> i32 {
        self.count += value;
        println!("{}: Added {}, new count is {}", self.name, value, self.count);
        self.count
    }
    
    /// Reset the counter to a specific value
    pub async fn reset(&mut self, value: i32) -> i32 {
        let old = self.count;
        self.count = value;
        println!("{}: Reset from {} to {}", self.name, old, self.count);
        old
    }
}

#[rustyray::main]
async fn main() -> Result<()> {
    println!("=== RustyRay Counter Example with Macros ===\n");
    
    // Create multiple counters with different constructors
    let counter1 = Counter::remote("Counter1".to_string(), 10).await?;
    let counter2 = Counter::remote_zero("Counter2".to_string()).await?;
    
    println!("Created two counters\n");
    
    // Operate on counter1
    println!("Operating on Counter1:");
    let val1 = counter1.increment().await?.get().await?;
    println!("  After increment: {}", val1);
    
    let val2 = counter1.add(5).await?.get().await?;
    println!("  After adding 5: {}", val2);
    
    let current = counter1.get().await?.get().await?;
    println!("  Current value: {}", current);
    
    // Operate on counter2
    println!("\nOperating on Counter2:");
    for i in 0..3 {
        let val = counter2.increment().await?.get().await?;
        println!("  Iteration {}: {}", i + 1, val);
    }
    
    let val3 = counter2.decrement().await?.get().await?;
    println!("  After decrement: {}", val3);
    
    // Reset counter2
    let old = counter2.reset(100).await?.get().await?;
    println!("  Reset from {} to 100", old);
    
    let final_val = counter2.get().await?.get().await?;
    println!("  Final value: {}", final_val);
    
    // Parallel operations
    println!("\nParallel operations on both counters:");
    let (r1, r2) = tokio::join!(
        counter1.add(10),
        counter2.add(20)
    );
    
    let v1 = r1?.get().await?;
    let v2 = r2?.get().await?;
    println!("  Counter1: {}, Counter2: {}", v1, v2);
    
    println!("\n✓ Counter example completed!");
    Ok(())
}