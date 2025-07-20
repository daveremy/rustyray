use rustyray::prelude::*;

#[rustyray::actor]
struct Calculator {
    base: i32,
}

#[rustyray::actor_methods]
impl Calculator {
    pub fn new(base: i32) -> Self {
        Calculator { base }
    }
    
    // This should fail - method without self
    pub fn add(x: i32, y: i32) -> i32 {
        x + y
    }
}

fn main() {}