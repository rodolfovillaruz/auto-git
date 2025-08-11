// src/event_handler.rs

use notify::event::{AccessKind, CreateKind, Event, EventKind, ModifyKind, RemoveKind, RenameMode};

/// A helper function to neatly print details about a batch of file system events.
/// This is the "action" that is performed after debouncing.
pub fn handle_events(events: &[Event]) {
    if events.is_empty() {
        return;
    }

    println!("\n=======================================================");
    println!("✅ DEBOUNCED ACTION! Processing {} events...", events.len());
    println!("=======================================================");

    for event in events {
        handle_single_event(event);
    }
    println!("-------------------------------------------------------\n");
}

/// Processes and prints a single file system event.
fn handle_single_event(event: &Event) {
    let path = event
        .paths
        .first()
        .map_or("N/A", |p| p.to_str().unwrap_or("Invalid UTF-8"));

    match &event.kind {
        EventKind::Create(CreateKind::File) => println!("[CREATE] File created: {}", path),
        EventKind::Create(CreateKind::Folder) => println!("[CREATE] Folder created: {}", path),
        EventKind::Modify(ModifyKind::Data(_)) => {
            println!("[MODIFY] File content changed: {}", path)
        }
        EventKind::Modify(ModifyKind::Name(RenameMode::To)) => {
            println!("[RENAME] Renamed/Moved to: {}", path)
        }
        EventKind::Modify(ModifyKind::Name(RenameMode::From)) => {
            println!("[RENAME] Renamed/Moved from: {}", path)
        }
        EventKind::Remove(RemoveKind::File) => println!("[REMOVE] File removed: {}", path),
        EventKind::Remove(RemoveKind::Folder) => println!("[REMOVE] Folder removed: {}", path),
        EventKind::Access(AccessKind::Close(_)) => println!("[ACCESS] File closed: {}", path),
        _ => println!("[OTHER] Event: {:?} on {}", event.kind, path),
    }
}
