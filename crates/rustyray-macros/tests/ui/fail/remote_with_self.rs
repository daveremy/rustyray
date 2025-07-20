use rustyray::prelude::*;

// Remote function with self parameter - should fail
struct Calculator;

impl Calculator {
    #[rustyray::remote]
    async fn compute(&self, x: i32, y: i32) -> i32 {
        x + y
    }
}

fn main() {}