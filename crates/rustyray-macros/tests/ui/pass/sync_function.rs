use rustyray_macros::remote;

#[remote]
fn sync_add(x: i32, y: i32) -> i32 {
    x + y
}

#[remote(num_cpus = 1.5)]
fn sync_with_resources(data: Vec<i32>) -> i32 {
    data.iter().sum()
}

fn main() {
    // Just verify it compiles
    let _ = sync_add;
    let _ = sync_with_resources;
}