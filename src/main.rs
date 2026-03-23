use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow};
use std::sync::Arc;
use std::thread;
use std::sync::mpsc;

mod ghostty;

const APP_ID: &str = "io.cmux.App";

fn main() {
    // Create tokio runtime before GTK app initialization
    let runtime = Arc::new(
        tokio::runtime::Runtime::new()
            .expect("Failed to create tokio runtime")
    );
    
    // Spawn the runtime in a separate thread to avoid blocking GTK
    let runtime_handle = runtime.handle().clone();
    thread::spawn(move || {
        // Keep the runtime alive for the duration of the application
        // This thread will just park itself after creating the runtime
        thread::park();
    });
    
    // Create a channel to bridge tokio to GTK main thread
    // Using std::sync::mpsc for now, but in production we'd use glib::MainContext::channel
    let (tx, rx) = mpsc::channel::<String>();
    
    // Use glib::idle_add to process messages in GTK main thread
    // This is the pattern for bridging async tasks to the main thread
    let rx = Arc::new(std::sync::Mutex::new(rx));
    glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
        if let Ok(rx) = rx.lock() {
            if let Ok(msg) = rx.try_recv() {
                println!("Received from tokio: {}", msg);
            }
        }
        glib::ControlFlow::Continue
    });
    
    // Store the sender for tokio tasks to use
    let tx_for_tokio = tx.clone();
    
    // Test the bridge with a simple message
    runtime_handle.spawn(async move {
        println!("Tokio runtime started successfully");
        
        // Simulate async work
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        
        // Send message to GTK main thread
        let _ = tx_for_tokio.send("Tokio runtime successfully bridged to GLib main loop".to_string());
        
        // Log that both runtimes are working
        println!("Tokio task completed - message sent to GLib main thread");
    });

    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(build_ui);
    app.run();
    
    // Shutdown the runtime when the app exits
    // Note: In production, we'd want a more graceful shutdown mechanism
}

fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("cmux")
        .default_width(800)
        .default_height(600)
        .build();

    let gl_area = ghostty::surface::create_surface(app);
    window.set_child(Some(&gl_area));

    window.present();
}
