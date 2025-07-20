use rustyray::prelude::*;

#[rustyray::actor]
struct MyActor {
    value: i32,
}

trait MyTrait {
    fn get_value(&self) -> i32;
}

// This should fail - actor_methods on trait impl
#[rustyray::actor_methods]
impl MyTrait for MyActor {
    fn get_value(&self) -> i32 {
        self.value
    }
}

fn main() {}