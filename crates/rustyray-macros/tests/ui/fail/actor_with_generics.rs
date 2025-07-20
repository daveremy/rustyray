use rustyray::prelude::*;

// Actor with generics - should fail
#[rustyray::actor]
struct GenericActor<T> {
    value: T,
}

fn main() {}