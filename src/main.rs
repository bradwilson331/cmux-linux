use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, gio};
use std::sync::Arc;
use std::thread;
use std::sync::mpsc;

mod ghostty;
mod workspace;
mod split_engine;

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

    // NON_UNIQUE bypasses DBus singleton check — required in environments where
    // cross-namespace DBus EXTERNAL auth deadlocks (e.g. NX/container sessions).
    let app = Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();

    eprintln!("cmux: GtkApplication created, connecting activate signal");
    app.connect_activate(build_ui);
    eprintln!("cmux: calling app.run()");
    let exit_code = app.run();
    eprintln!("cmux: app.run() returned");
    
    // Shutdown the runtime when the app exits
    // Note: In production, we'd want a more graceful shutdown mechanism
}

fn build_ui(app: &Application) {
    eprintln!("cmux: build_ui called — creating window");
    let window = ApplicationWindow::builder()
        .application(app)
        .title("cmux")
        .default_width(800)
        .default_height(600)
        .build();

    eprintln!("cmux: ApplicationWindow created, creating GLArea surface");
    let gl_area = ghostty::surface::create_surface(app);
    window.set_child(Some(&gl_area));
    eprintln!("cmux: window.present() about to be called");
    window.present();
    eprintln!("cmux: window.present() returned");
}
