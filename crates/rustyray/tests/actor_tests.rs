//! Integration tests for actor macros
#![cfg(test)]

use rustyray::prelude::*;

#[rustyray::actor]
struct TestActor {
    value: i32,
    name: String,
}

#[rustyray::actor_methods]
impl TestActor {
    pub fn new(name: String) -> Self {
        TestActor { value: 0, name }
    }
    
    pub async fn increment(&mut self) -> i32 {
        self.value += 1;
        self.value
    }
    
    pub fn get_value(&self) -> i32 {
        self.value
    }
    
    pub async fn add(&mut self, amount: i32) -> i32 {
        self.value += amount;
        self.value
    }
}

#[tokio::test]
async fn test_actor_basic_operations() {
    // Initialize runtime if needed
    let _ = rustyray::runtime::init();
    
    // Create actor
    let actor = TestActor::remote("test".to_string()).await.unwrap();
    
    // Test increment
    let val1 = actor.increment().await.unwrap().get().await.unwrap();
    assert_eq!(val1, 1);
    
    // Test add
    let val2 = actor.add(5).await.unwrap().get().await.unwrap();
    assert_eq!(val2, 6);
    
    // Test get_value
    let current = actor.get_value().await.unwrap().get().await.unwrap();
    assert_eq!(current, 6);
}

#[rustyray::actor]
struct MultiConstructorActor {
    mode: String,
    value: i32,
}

#[rustyray::actor_methods]
impl MultiConstructorActor {
    pub fn default() -> Self {
        MultiConstructorActor {
            mode: "default".to_string(),
            value: 0,
        }
    }
    
    pub fn with_value(value: i32) -> Self {
        MultiConstructorActor {
            mode: "custom".to_string(),
            value,
        }
    }
    
    pub async fn get_info(&self) -> (String, i32) {
        (self.mode.clone(), self.value)
    }
}

#[tokio::test]
async fn test_multiple_constructors() {
    let _ = rustyray::runtime::init();
    
    // Test default constructor
    let actor1 = MultiConstructorActor::remote_default().await.unwrap();
    let (mode1, val1) = actor1.get_info().await.unwrap().get().await.unwrap();
    assert_eq!(mode1, "default");
    assert_eq!(val1, 0);
    
    // Test custom constructor
    let actor2 = MultiConstructorActor::remote_with_value(42).await.unwrap();
    let (mode2, val2) = actor2.get_info().await.unwrap().get().await.unwrap();
    assert_eq!(mode2, "custom");
    assert_eq!(val2, 42);
}