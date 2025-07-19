use rustyray_macros::remote;

#[remote]
const fn invalid_const() -> i32 {
    42
}

fn main() {}