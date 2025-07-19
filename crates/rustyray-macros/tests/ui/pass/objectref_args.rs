use rustyray_macros::remote;
use rustyray_core::ObjectRef;

#[remote]
async fn with_one_ref(prev: ObjectRef<i32>, x: i32) -> i32 {
    prev.get().await.unwrap() + x
}

#[remote]
async fn with_multiple_refs(ref1: ObjectRef<i32>, ref2: ObjectRef<String>, y: f64) -> f64 {
    let val1 = ref1.get().await.unwrap();
    let val2 = ref2.get().await.unwrap();
    val1 as f64 + val2.len() as f64 + y
}

#[remote]
fn sync_with_ref(data: ObjectRef<Vec<u8>>) -> usize {
    // In a real implementation, this would need to be async
    // For now, just return a dummy value
    42
}

fn main() {
    // Just verify it compiles
    let _ = with_one_ref;
    let _ = with_multiple_refs;
    let _ = sync_with_ref;
}