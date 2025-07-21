//! Tests for multi-argument functions with ObjectRef dependencies
//! This specifically tests the fix for the argument serialization bug

#[cfg(test)]
mod tests {
    use crate::actor::ActorSystem;
    use crate::error::Result;
    use crate::object_ref::ObjectRef;
    use crate::task::{TaskBuilder, TaskSystem};
    use crate::task_function;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_multi_arg_with_objectref() -> Result<()> {
        // Create systems
        let actor_system = Arc::new(ActorSystem::new());
        let task_system = Arc::new(TaskSystem::new(actor_system.clone()));

        // Register functions
        task_system.register_function(
            "add3",
            task_function!(|x: i32, y: i32, z: i32| async move {
                println!("   Task: Adding {} + {} + {}", x, y, z);
                Ok::<i32, crate::error::RustyRayError>(x + y + z)
            }),
        )?;

        task_system.register_function(
            "multiply",
            task_function!(|x: i32, y: i32| async move {
                println!("   Task: Multiplying {} * {}", x, y);
                Ok::<i32, crate::error::RustyRayError>(x * y)
            }),
        )?;

        // Test 1: Direct values with multiple arguments
        println!("Test 1: Three direct arguments");
        let result_ref: ObjectRef<i32> = TaskBuilder::new("add3")
            .arg(10)
            .arg(20)
            .arg(30)
            .submit(&task_system)
            .await?;
        let result = result_ref.get().await?;
        assert_eq!(result, 60);
        println!("   Result: {} ✓", result);

        // Test 2: Mix of direct values and ObjectRef
        println!("\nTest 2: Mixed direct and ObjectRef arguments");

        // First compute 5 * 6 = 30
        let multiply_ref: ObjectRef<i32> = TaskBuilder::new("multiply")
            .arg(5)
            .arg(6)
            .submit(&task_system)
            .await?;

        // Then add 10 + multiply_result + 20
        let add_ref: ObjectRef<i32> = TaskBuilder::new("add3")
            .arg(10)
            .arg_ref(&multiply_ref) // This is an ObjectRef dependency!
            .arg(20)
            .submit(&task_system)
            .await?;

        let final_result = add_ref.get().await?;
        assert_eq!(final_result, 60); // 10 + 30 + 20 = 60
        println!("   Final result: {} ✓", final_result);

        // Test 3: Multiple ObjectRef dependencies
        println!("\nTest 3: Multiple ObjectRef dependencies");

        // Create some values using put()
        let val1 = task_system.put(100).await?;
        let val2 = task_system.put(200).await?;
        let val3 = task_system.put(300).await?;

        // Add them all together
        let sum_ref: ObjectRef<i32> = TaskBuilder::new("add3")
            .arg_ref(&val1)
            .arg_ref(&val2)
            .arg_ref(&val3)
            .submit(&task_system)
            .await?;

        let sum = sum_ref.get().await?;
        assert_eq!(sum, 600); // 100 + 200 + 300 = 600
        println!("   Sum of ObjectRefs: {} ✓", sum);

        // Test 4: Chain of ObjectRef dependencies
        println!("\nTest 4: Chain of ObjectRef dependencies");

        // First: 3 * 4 = 12
        let step1 = TaskBuilder::new("multiply")
            .arg(3)
            .arg(4)
            .submit::<i32>(&task_system)
            .await?;

        // Second: 5 * 6 = 30
        let step2 = TaskBuilder::new("multiply")
            .arg(5)
            .arg(6)
            .submit::<i32>(&task_system)
            .await?;

        // Third: step1 * 2 = 24
        let step3 = TaskBuilder::new("multiply")
            .arg_ref(&step1)
            .arg(2)
            .submit::<i32>(&task_system)
            .await?;

        // Final: step3 + step2 + 10 = 24 + 30 + 10 = 64
        let final_ref = TaskBuilder::new("add3")
            .arg_ref(&step3)
            .arg_ref(&step2)
            .arg(10)
            .submit::<i32>(&task_system)
            .await?;

        let chain_result = final_ref.get().await?;
        assert_eq!(chain_result, 64);
        println!("   Chain result: {} ✓", chain_result);

        println!("\nAll tests passed! Multi-argument functions with ObjectRef dependencies work correctly.");

        // Cleanup
        if let Ok(task_system) = Arc::try_unwrap(task_system) {
            task_system.shutdown().await?;
        }
        if let Ok(actor_system) = Arc::try_unwrap(actor_system) {
            actor_system.shutdown().await?;
        }

        Ok(())
    }
}
