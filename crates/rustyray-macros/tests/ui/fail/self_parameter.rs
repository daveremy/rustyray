use rustyray_macros::remote;

struct MyStruct;

impl MyStruct {
    #[remote]
    async fn invalid_method(&self) -> i32 {
        42
    }
}

fn main() {}