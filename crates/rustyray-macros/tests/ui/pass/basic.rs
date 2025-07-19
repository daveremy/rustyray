use rustyray_macros::remote;

#[remote]
async fn valid_async(x: i32, y: i32) -> i32 {
    x + y
}

#[remote(num_cpus = 2.0)]
async fn with_resources(data: String) -> String {
    data.to_uppercase()
}

fn main() {
    // Just verify it compiles
    let _ = valid_async;
    let _ = with_resources;
}