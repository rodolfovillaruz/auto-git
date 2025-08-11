// src/debouncer.rs

use crate::event_handler;
use notify::Event;
use std::time::Duration;
use tokio::sync::mpsc::Receiver;
use tokio::time::{Instant, sleep};

/// The core debouncing logic.
///
/// It receives events, and if no new event arrives within `debounce_duration`,
/// it processes the accumulated events.
pub async fn debouncer(mut rx: Receiver<Event>, debounce_duration: Duration) {
    let mut last_event_time: Option<Instant> = None;
    let mut accumulated_events: Vec<Event> = Vec::new();

    loop {
        // Wait for the first event in a new batch
        match rx.recv().await {
            Some(event) => {
                println!("-> Event received. Starting/resetting debounce timer...");
                accumulated_events.push(event);
                last_event_time = Some(Instant::now());
            }
            None => break, // Channel closed
        }

        // Debounce loop: continue consuming events until the timer expires
        while let Some(last_time) = last_event_time {
            let timeout = debounce_duration.saturating_sub(last_time.elapsed());

            tokio::select! {
                // Timer expires: process events
                _ = sleep(timeout) => {
                    // Pass the collected events to the handler
                    event_handler::handle_events(&accumulated_events);

                    // Reset state for the next batch
                    accumulated_events.clear();
                    last_event_time = None;
                    break; // Exit inner loop to wait for a new "first" event
                }

                // New event arrives: reset timer
                Some(event) = rx.recv() => {
                    println!("-> Event received. Resetting debounce timer...");
                    accumulated_events.push(event);
                    last_event_time = Some(Instant::now());
                }

                else => break, // Channel closed
            }
        }
    }
}
