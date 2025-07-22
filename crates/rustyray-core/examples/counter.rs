use async_trait::async_trait;
use rustyray_core::actor::Actor;
use rustyray_core::error::Result;
use rustyray_core::runtime;
use std::any::Any;

/// A simple counter actor that maintains internal state.
struct Counter {
    count: i32,
}

/// Messages that the Counter actor can handle.
#[derive(Debug)]
enum CounterMessage {
    Increment,
    Decrement,
    Get,
}

/// Response from the Counter actor.
#[derive(Debug)]
enum CounterResponse {
    Count(i32),
}

#[async_trait]
impl Actor for Counter {
    async fn handle(&mut self, msg: Box<dyn Any + Send>) -> Result<Box<dyn Any + Send>> {
        // Try to downcast the message to our expected type
        if let Ok(msg) = msg.downcast::<CounterMessage>() {
            match *msg {
                CounterMessage::Increment => {
                    self.count += 1;
                    println!("Counter incremented to: {}", self.count);
                    Ok(Box::new(CounterResponse::Count(self.count)))
                }
                CounterMessage::Decrement => {
                    self.count -= 1;
                    println!("Counter decremented to: {}", self.count);
                    Ok(Box::new(CounterResponse::Count(self.count)))
                }
                CounterMessage::Get => {
                    println!("Counter value requested: {}", self.count);
                    Ok(Box::new(CounterResponse::Count(self.count)))
                }
            }
        } else {
            Err(rustyray_core::error::RustyRayError::InvalidMessage)
        }
    }

    async fn on_start(&mut self) -> Result<()> {
        println!("Counter actor started with initial value: {}", self.count);
        Ok(())
    }

    async fn on_stop(&mut self) -> Result<()> {
        println!("Counter actor stopping with final value: {}", self.count);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Starting Counter actor example...\n");

    // Initialize the runtime
    runtime::init()?;
    let rt = runtime::global()?;
    let system = rt.actor_system();

    // Create a counter actor
    let counter = Counter { count: 0 };
    let counter_ref = system.create_actor(counter).await?;
    println!("Created counter actor with ID: {}", counter_ref.id());

    // Send some fire-and-forget messages
    println!("\n--- Sending increment messages ---");
    counter_ref
        .send(Box::new(CounterMessage::Increment))
        .await?;
    counter_ref
        .send(Box::new(CounterMessage::Increment))
        .await?;
    counter_ref
        .send(Box::new(CounterMessage::Increment))
        .await?;

    // Call and wait for response
    println!("\n--- Getting current count ---");
    let response = counter_ref.call(Box::new(CounterMessage::Get)).await?;
    if let Ok(response) = response.downcast::<CounterResponse>() {
        match *response {
            CounterResponse::Count(count) => println!("Current count via call: {count}"),
        }
    }

    // Send more messages
    println!("\n--- Sending mixed messages ---");
    counter_ref
        .send(Box::new(CounterMessage::Decrement))
        .await?;
    counter_ref
        .send(Box::new(CounterMessage::Increment))
        .await?;
    counter_ref
        .send(Box::new(CounterMessage::Increment))
        .await?;

    // Get final count
    println!("\n--- Getting final count ---");
    let response = counter_ref.call(Box::new(CounterMessage::Get)).await?;
    if let Ok(response) = response.downcast::<CounterResponse>() {
        match *response {
            CounterResponse::Count(count) => println!("Final count via call: {count}"),
        }
    }

    // Shutdown the system
    println!("\n--- Shutting down ---");
    runtime::shutdown()?;

    println!("\nExample completed successfully!");
    Ok(())
}
