mod models;
mod workflow;

use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

use models::event::Event;
use workflow::storage::{EventRepository, UserRepository, WorkflowRepository};
use workflow::PostgresStorage;
use workflow::user_activity_workflow;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let storage = PostgresStorage::new("postgres://postgres:postgres@localhost/ariadne").await?;
    storage.setup_database().await?;

    // Create a test user
    let user_id = Uuid::new_v4();
    storage.create_user(user_id, "test user").await?;

    // Create and start workflow
    let mut workflow = user_activity_workflow::create_demo_workflow();
    workflow.user_id = user_id; // Set the user ID for the workflow
    storage.save_workflow(&workflow).await?;

    // Process user activity event
    let event = Event::UserActivity;
    storage.save_event(user_id, &event).await?;
    workflow.process_event(&event);
    storage.save_workflow(&workflow).await?;

    // Simulate time passing
    sleep(Duration::from_secs(1)).await;

    // Process timer event
    let event = Event::Timer {
        timer_id: "1".to_string(),
    };
    storage.save_event(user_id, &event).await?;
    workflow.process_event(&event);
    storage.save_workflow(&workflow).await?;

    // Print database contents
    println!("\nDatabase contents:");
    println!("Events:");
    let events = storage.get_all_events().await?;
    for (id, user_id, event_type, event_data, created_at) in events {
        println!(
            "  ID: {}, User: {}, Type: {}, Data: {}, Created: {}",
            id, user_id, event_type, event_data, created_at
        );
    }

    println!("\nWorkflows:");
    let workflows = storage.get_all_workflows().await?;
    for (id, user_id, name, status) in workflows {
        println!(
            "  ID: {}, User: {}, Name: {}, Status: {}",
            id, user_id, name, status
        );

        if let Some(workflow) = storage.load_workflow(user_id, id).await? {
            println!("  Nodes:");
            for (i, node) in workflow.nodes.iter().enumerate() {
                println!("    {}: {} - {:?}", i, node.name, node.status);
            }
        }
    }

    println!("All done!");
    Ok(())
}
