mod models;
mod workflow;

use uuid::Uuid;

use models::event::Event;
use workflow::storage::{EventRepository, UserRepository, WorkflowRepository};
use workflow::PostgresStorage;
use workflow::user_activity_workflow;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let storage = PostgresStorage::new("postgres://postgres:postgres@localhost/ariadne").await?;

    // Create a test user
    let user_id: Uuid = Uuid::new_v4();
    storage.create_user(user_id, "test user").await?;

    // Create and start workflow
    let mut workflow = user_activity_workflow::create_demo_workflow();
    workflow.user_id = user_id;
    storage.save_workflow(&workflow).await?;

    // Process user activity event
    let event = Event::UserActivity;
    storage.save_event(user_id, &event).await?;
    workflow.process_event(&event);
    storage.save_workflow(&workflow).await?;

    // Load workflow from database to test serialization
    let mut loaded_workflow = storage.load_workflow(user_id, workflow.id).await?
        .expect("Workflow should exist in database");

    // Compare workflows
    println!("\nComparing workflows:");
    println!("Original workflow ID: {}", workflow.id);
    println!("Loaded workflow ID: {}", loaded_workflow.id);
    println!("Original workflow status: {:?}", workflow.status);
    println!("Loaded workflow status: {:?}", loaded_workflow.status);
    
    println!("\nOriginal workflow nodes:");
    for (i, node) in workflow.nodes.iter().enumerate() {
        println!("  {}: {} - {:?}", i, node.name, node.status);
    }
    
    println!("\nLoaded workflow nodes:");
    for (i, node) in loaded_workflow.nodes.iter().enumerate() {
        println!("  {}: {} - {:?}", i, node.name, node.status);
    }

    // Process timer event on loaded workflow
    let timer_event = Event::Timer { timer_id: "1".to_string() };
    storage.save_event(user_id, &timer_event).await?;
    loaded_workflow.process_event(&timer_event);
    storage.save_workflow(&loaded_workflow).await?;

    println!("\nAfter timer event:");
    for (i, node) in loaded_workflow.nodes.iter().enumerate() {
        println!("  {}: {} - {:?}", i, node.name, node.status);
    }

    println!("\nAll done!");
    Ok(())
}
