//! Human-readable output formatters for cmux CLI responses.
//!
//! Handles color support (D-07), list formatting with active markers (D-08),
//! and mutation success messages (D-09).

use serde_json::Value;
use std::io::IsTerminal;

/// Determine whether to use color output based on the --color flag value.
pub fn use_color(color_flag: &str) -> bool {
    match color_flag {
        "always" => true,
        "never" => false,
        _ => std::io::stdout().is_terminal(),
    }
}

fn green(s: &str, color: bool) -> String {
    if color {
        format!("\x1b[1;32m{}\x1b[0m", s)
    } else {
        s.to_string()
    }
}

fn dim(s: &str, color: bool) -> String {
    if color {
        format!("\x1b[2m{}\x1b[0m", s)
    } else {
        s.to_string()
    }
}

/// Format a workspace list response with active marker.
pub fn format_workspace_list(result: &Value, color: bool) -> String {
    let workspaces = match result.get("workspaces").and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => return format_fallback(result),
    };
    if workspaces.is_empty() {
        return "No workspaces".to_string();
    }
    let mut lines = Vec::new();
    for (i, ws) in workspaces.iter().enumerate() {
        let selected = ws.get("selected").and_then(|v| v.as_bool()).unwrap_or(false);
        let title = ws
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("untitled");
        let pane_count = ws
            .get("pane_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let marker = if selected { "*" } else { " " };
        let line = format!(
            "{} {}: {} ({} pane{})",
            marker,
            i + 1,
            title,
            pane_count,
            if pane_count == 1 { "" } else { "s" }
        );
        if selected && color {
            lines.push(green(&line, true));
        } else {
            lines.push(line);
        }
    }
    lines.join("\n")
}

/// Format a surface list response with focused marker.
pub fn format_surface_list(result: &Value, color: bool) -> String {
    let surfaces = match result.get("surfaces").and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => return format_fallback(result),
    };
    if surfaces.is_empty() {
        return "No surfaces".to_string();
    }
    let mut lines = Vec::new();
    for surface in surfaces {
        let focused = surface
            .get("focused")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let id = surface
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let id_short = &id[..id.len().min(8)];
        let title = surface
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let marker = if focused { "*" } else { " " };
        let line = if title.is_empty() {
            format!("{} {}", marker, id_short)
        } else {
            format!("{} {} ({})", marker, id_short, title)
        };
        if focused && color {
            lines.push(green(&line, true));
        } else {
            lines.push(line);
        }
    }
    lines.join("\n")
}

/// Format a pane list response with focused marker.
pub fn format_pane_list(result: &Value, color: bool) -> String {
    let panes = match result.get("panes").and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => return format_fallback(result),
    };
    if panes.is_empty() {
        return "No panes".to_string();
    }
    let mut lines = Vec::new();
    for pane in panes {
        let focused = pane
            .get("focused")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let id = pane
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let id_short = &id[..id.len().min(8)];
        let title = pane.get("title").and_then(|v| v.as_str()).unwrap_or("");
        let marker = if focused { "*" } else { " " };
        let line = if title.is_empty() {
            format!("{} {}", marker, id_short)
        } else {
            format!("{} {} ({})", marker, id_short, title)
        };
        if focused && color {
            lines.push(green(&line, true));
        } else {
            lines.push(line);
        }
    }
    lines.join("\n")
}

/// Format a window list response.
pub fn format_window_list(result: &Value, color: bool) -> String {
    let windows = match result.get("windows").and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => return format_fallback(result),
    };
    if windows.is_empty() {
        return "No windows".to_string();
    }
    let mut lines = Vec::new();
    for (i, win) in windows.iter().enumerate() {
        let focused = win
            .get("focused")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let title = win.get("title").and_then(|v| v.as_str()).unwrap_or("");
        let marker = if focused { "*" } else { " " };
        let line = format!("{} {}: {}", marker, i + 1, title);
        if focused && color {
            lines.push(green(&line, true));
        } else {
            lines.push(line);
        }
    }
    lines.join("\n")
}

/// Format identify response.
fn format_identify(result: &Value) -> String {
    let version = result
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let platform = result
        .get("platform")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let pid = result.get("pid").and_then(|v| v.as_u64());
    match pid {
        Some(p) => format!("cmux {} ({}) pid {}", version, platform, p),
        None => format!("cmux {} ({})", version, platform),
    }
}

/// Format capabilities response.
fn format_capabilities(result: &Value, color: bool) -> String {
    let methods = match result.get("methods").and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => return format_fallback(result),
    };
    let mut lines = vec![format!("{} methods available:", methods.len())];
    for m in methods {
        let name = m.as_str().unwrap_or("?");
        lines.push(format!("  {}", dim(name, color)));
    }
    lines.join("\n")
}

/// Format notification list response.
fn format_notification_list(result: &Value, color: bool) -> String {
    let notifications = match result.get("notifications").and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => return format_fallback(result),
    };
    if notifications.is_empty() {
        return "No notifications".to_string();
    }
    let mut lines = Vec::new();
    for n in notifications {
        let id = n
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let id_short = &id[..id.len().min(8)];
        let attention = n
            .get("attention")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let marker = if attention { "!" } else { " " };
        let line = format!("{} {}", marker, id_short);
        if attention && color {
            lines.push(green(&line, true));
        } else {
            lines.push(line);
        }
    }
    lines.join("\n")
}

/// Format a mutation command result with a success message.
pub fn format_mutation(command_name: &str, result: &Value) -> String {
    let id = result
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let title = result
        .get("title")
        .or_else(|| result.get("name"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    match command_name {
        "workspace.create" => {
            if title.is_empty() {
                format!("Created workspace: {}", id)
            } else {
                format!("Created workspace: {} ({})", title, id)
            }
        }
        "workspace.close" => format!("Closed workspace: {}", id),
        "workspace.rename" => {
            let name = result
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or(title);
            format!("Renamed workspace {} to: {}", id, name)
        }
        "surface.split" => format!("Split created: {}", id),
        "surface.close" => format!("Closed surface: {}", id),
        _ => String::new(),
    }
}

/// Format a command response for human-readable output.
///
/// If `json_mode` is true, returns raw JSON (D-06).
/// Otherwise, picks the appropriate formatter based on the method name.
pub fn format_response(method: &str, result: &Value, json_mode: bool, color: bool) -> String {
    if json_mode {
        return serde_json::to_string_pretty(result).unwrap_or_default();
    }

    match method {
        "workspace.list" => format_workspace_list(result, color),
        "workspace.current" => {
            let title = result
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let id = result
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            format!("{} ({})", title, id)
        }
        "surface.list" => format_surface_list(result, color),
        "pane.list" => format_pane_list(result, color),
        "window.list" => format_window_list(result, color),
        "window.current" => {
            let title = result
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            title.to_string()
        }
        "system.ping" => {
            result
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("pong")
                .to_string()
        }
        "system.identify" => format_identify(result),
        "system.capabilities" => format_capabilities(result, color),
        "notification.list" => format_notification_list(result, color),
        "debug.layout" => serde_json::to_string_pretty(result).unwrap_or_default(),

        // Mutation commands: show success message
        "workspace.create" | "workspace.close" | "workspace.rename" | "surface.split"
        | "surface.close" => {
            let msg = format_mutation(method, result);
            if msg.is_empty() {
                format_fallback(result)
            } else {
                msg
            }
        }

        // Default: pretty-print JSON for uncommon commands
        _ => format_fallback(result),
    }
}

/// Fallback: pretty-print JSON.
fn format_fallback(result: &Value) -> String {
    serde_json::to_string_pretty(result).unwrap_or_default()
}
