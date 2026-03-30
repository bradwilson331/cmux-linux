// src/socket/handlers.rs — GTK main thread command dispatch

use crate::socket::commands::SocketCommand;
use gtk4::prelude::*;
use serde_json::{json, Value};

/// Build a success response with the given result payload.
fn ok(req_id: Value, result: Value) -> Value {
    json!({"id": req_id, "ok": true, "result": result})
}

/// Build an error response.
fn err(req_id: Value, code: &str, message: &str) -> Value {
    json!({"id": req_id, "ok": false, "error": {"code": code, "message": message}})
}

/// Resolve a surface_ref string ("surface:N" or UUID) to a UUID string.
/// Returns Ok(uuid_string) or Err((error_message, available_refs)).
fn resolve_surface_ref(
    surface_ref: &str,
    refs: &std::collections::HashMap<u32, String>,
) -> Result<String, (String, Vec<String>)> {
    if let Some(n_str) = surface_ref.strip_prefix("surface:") {
        if let Ok(n) = n_str.parse::<u32>() {
            if let Some(uuid) = refs.get(&n) {
                return Ok(uuid.clone());
            }
            let available: Vec<String> = refs.keys()
                .map(|k| format!("surface:{}", k))
                .collect();
            return Err((format!("surface:{} not found", n), available));
        }
    }
    // Treat as UUID directly
    Ok(surface_ref.to_string())
}

/// Dispatch a SocketCommand on the GTK main thread.
/// SOCK-05: Only focus-intent commands (workspace.select, workspace.next/previous/last,
/// pane.focus, pane.last, surface.focus) may call grab_active_focus() or focus_active_surface().
#[allow(unused_variables)]
pub fn handle_socket_command(
    cmd: SocketCommand,
    state: &crate::app_state::AppStateRef,
) {
    match cmd {
        // -- system.* --
        SocketCommand::Ping { req_id, resp_tx } => {
            let _ = resp_tx.send(ok(req_id, json!({"pong": true})));
        }

        SocketCommand::Identify { req_id, resp_tx } => {
            let socket_path = crate::socket::socket_path().to_string_lossy().to_string();
            let _ = resp_tx.send(ok(req_id, json!({
                "version": env!("CARGO_PKG_VERSION"),
                "platform": "linux",
                "socket_path": socket_path,
            })));
        }

        SocketCommand::Capabilities { req_id, resp_tx } => {
            let methods: Vec<&str> = vec![
                "system.ping", "system.identify", "system.capabilities",
                "workspace.list", "workspace.current", "workspace.create",
                "workspace.select", "workspace.close", "workspace.rename",
                "workspace.next", "workspace.previous", "workspace.last", "workspace.reorder",
                "surface.list", "surface.split", "surface.focus", "surface.close",
                "surface.send_text", "surface.send_key", "surface.read_text",
                "surface.health", "surface.refresh",
                "pane.list", "pane.focus", "pane.last",
                "window.list", "window.current",
                "notification.list", "notification.clear",
                // Browser lifecycle + streaming
                "browser.open", "browser.close", "browser.list",
                "browser.stream.enable", "browser.stream.disable",
                "browser.snapshot", "browser.screenshot",
                // P0: navigation
                "browser.navigate", "browser.goto",
                "browser.back", "browser.forward", "browser.reload",
                // P0: interaction
                "browser.click", "browser.dblclick", "browser.type", "browser.fill",
                "browser.press", "browser.keydown", "browser.keyup",
                "browser.hover", "browser.focus",
                "browser.check", "browser.uncheck", "browser.select",
                "browser.scroll", "browser.scroll_into_view",
                "browser.drag", "browser.upload", "browser.download", "browser.pdf",
                // P0: evaluation + waiting
                "browser.eval", "browser.wait",
                // P0: getters
                "browser.get.url", "browser.get.title", "browser.get.text",
                "browser.get.html", "browser.get.value", "browser.get.attr",
                "browser.get.count", "browser.get.box", "browser.get.styles",
                // P0: state checks
                "browser.is.visible", "browser.is.enabled", "browser.is.checked",
                // P1: locators
                "browser.find.role", "browser.find.text", "browser.find.label",
                "browser.find.placeholder", "browser.find.alt", "browser.find.title",
                "browser.find.testid", "browser.find.nth", "browser.find.first",
                "browser.find.last",
                // P1: frames, dialogs, console, errors
                "browser.frame.select", "browser.frame.main",
                "browser.dialog.accept", "browser.dialog.dismiss",
                "browser.console.list", "browser.errors.list",
                "browser.highlight",
                "browser.state.save", "browser.state.load",
                // Debug
                "debug.layout", "debug.type",
            ];
            let _ = resp_tx.send(ok(req_id, json!({"methods": methods})));
        }

        // -- workspace.* --
        SocketCommand::WorkspaceList { req_id, resp_tx } => {
            // SOCK-05: No focus side effects.
            let s = state.borrow();
            let list: Vec<Value> = s.workspaces.iter().enumerate().map(|(i, ws)| {
                json!({
                    "index": i,
                    "id": ws.uuid.to_string(),
                    "title": ws.name,
                    "selected": i == s.active_index,
                })
            }).collect();
            let _ = resp_tx.send(ok(req_id, json!({"workspaces": list})));
        }

        SocketCommand::WorkspaceCurrent { req_id, resp_tx } => {
            // SOCK-05: No focus side effects.
            let s = state.borrow();
            match s.active_workspace() {
                Some(ws) => {
                    let _ = resp_tx.send(ok(req_id, json!({
                        "uuid": ws.uuid.to_string(),
                        "name": ws.name,
                    })));
                }
                None => {
                    let _ = resp_tx.send(err(req_id, "no_workspace", "no active workspace"));
                }
            }
        }

        SocketCommand::WorkspaceCreate { req_id, remote_target, resp_tx } => {
            if let Some(target) = remote_target {
                // SSH workspace creation per D-13, D-15
                // Create per-workspace bridge for SSH I/O routing
                let (write_tx, write_rx) = tokio::sync::mpsc::unbounded_channel();
                let (output_tx, _output_rx) = tokio::sync::mpsc::unbounded_channel();
                let bridge = std::sync::Arc::new(crate::ssh::bridge::SshBridge::new(write_tx, write_rx, output_tx));
                let id = state.borrow_mut().create_remote_workspace(target.clone(), &bridge);
                // Store bridge on AppState for later access
                state.borrow_mut().workspace_bridges.insert(id, bridge.clone());
                let uuid_str = {
                    let s = state.borrow();
                    s.workspaces.iter()
                        .find(|ws| ws.id == id)
                        .map(|ws| ws.uuid.to_string())
                        .unwrap_or_default()
                };
                // Spawn SSH lifecycle task using the runtime_handle stored on AppState
                let ssh_tx = state.borrow().ssh_event_tx.clone();
                let rt_handle = state.borrow().runtime_handle.clone();
                if let (Some(tx), Some(rt)) = (ssh_tx, rt_handle) {
                    let handle = rt.spawn(crate::ssh::tunnel::run_ssh_lifecycle(id, target, tx, bridge));
                    state.borrow_mut().ssh_task_handles.insert(id, handle);
                }
                let _ = resp_tx.send(ok(req_id, json!({"uuid": uuid_str, "remote": true})));
            } else {
                // Local workspace (existing behavior)
                let id = state.borrow_mut().create_workspace();
                let s = state.borrow();
                let uuid_str = s.workspaces.iter()
                    .find(|ws| ws.id == id)
                    .map(|ws| ws.uuid.to_string())
                    .unwrap_or_default();
                let _ = resp_tx.send(ok(req_id, json!({"uuid": uuid_str})));
            }
        }

        SocketCommand::WorkspaceSelect { req_id, id, resp_tx } => {
            // SOCK-05: workspace.select IS a focus-intent command.
            let idx = {
                let s = state.borrow();
                s.workspaces.iter().position(|ws| ws.uuid.to_string() == id)
            };
            match idx {
                Some(i) => {
                    state.borrow_mut().switch_to_index(i);
                    let _ = resp_tx.send(ok(req_id, json!({})));
                }
                None => {
                    let _ = resp_tx.send(err(req_id, "not_found", "workspace not found"));
                }
            }
        }

        SocketCommand::WorkspaceClose { req_id, id, resp_tx } => {
            // SOCK-05: No focus side effects (close_workspace adjusts index internally).
            let idx = {
                let s = state.borrow();
                s.workspaces.iter().position(|ws| ws.uuid.to_string() == id)
            };
            match idx {
                Some(i) => {
                    let closed = state.borrow_mut().close_workspace(i);
                    if closed {
                        let _ = resp_tx.send(ok(req_id, json!({})));
                    } else {
                        let _ = resp_tx.send(err(req_id, "last_workspace", "cannot close the last workspace"));
                    }
                }
                None => {
                    let _ = resp_tx.send(err(req_id, "not_found", "workspace not found"));
                }
            }
        }

        SocketCommand::WorkspaceRename { req_id, id, name, resp_tx } => {
            // SOCK-05: No focus side effects. Find workspace by uuid, switch to it
            // (rename_active requires the target to be active), then rename.
            let idx = {
                let s = state.borrow();
                s.workspaces.iter().position(|ws| ws.uuid.to_string() == id)
            };
            match idx {
                Some(i) => {
                    let mut s = state.borrow_mut();
                    let prev_active = s.active_index;
                    s.switch_to_index(i);
                    s.rename_active(name);
                    // Restore previous active index to avoid focus side effect.
                    s.switch_to_index(prev_active);
                    drop(s);
                    let _ = resp_tx.send(ok(req_id, json!({})));
                }
                None => {
                    let _ = resp_tx.send(err(req_id, "not_found", "workspace not found"));
                }
            }
        }

        SocketCommand::WorkspaceNext { req_id, resp_tx } => {
            // SOCK-05: focus-intent command.
            state.borrow_mut().switch_next();
            let _ = resp_tx.send(ok(req_id, json!({})));
        }

        SocketCommand::WorkspacePrev { req_id, resp_tx } => {
            // SOCK-05: focus-intent command.
            state.borrow_mut().switch_prev();
            let _ = resp_tx.send(ok(req_id, json!({})));
        }

        SocketCommand::WorkspaceLast { req_id, resp_tx } => {
            // SOCK-05: focus-intent command.
            // "Last" = most recently visited; for now same as prev (Phase 4 can track history).
            state.borrow_mut().switch_prev();
            let _ = resp_tx.send(ok(req_id, json!({})));
        }

        SocketCommand::WorkspaceReorder { req_id, id, position, resp_tx } => {
            // SOCK-05: No focus side effects.
            let mut s = state.borrow_mut();
            let idx = s.workspaces.iter().position(|ws| ws.uuid.to_string() == id);
            match idx {
                Some(from) => {
                    let to = position.min(s.workspaces.len().saturating_sub(1));
                    let ws = s.workspaces.remove(from);
                    let engine = s.split_engines.remove(from);
                    s.workspaces.insert(to, ws);
                    s.split_engines.insert(to, engine);
                    // Adjust active_index after reorder.
                    if from == s.active_index {
                        s.active_index = to;
                    } else if from < s.active_index && to >= s.active_index {
                        s.active_index -= 1;
                    } else if from > s.active_index && to <= s.active_index {
                        s.active_index += 1;
                    }
                    drop(s);
                    let _ = resp_tx.send(ok(req_id, json!({})));
                }
                None => {
                    drop(s);
                    let _ = resp_tx.send(err(req_id, "not_found", "workspace not found"));
                }
            }
        }

        // -- window.* --
        SocketCommand::WindowList { req_id, resp_tx } => {
            // SOCK-05: No focus side effects.
            let workspace_count = state.borrow().workspaces.len();
            let _ = resp_tx.send(ok(req_id, json!({
                "windows": [{"id": "main", "workspaces": workspace_count}]
            })));
        }

        SocketCommand::WindowCurrent { req_id, resp_tx } => {
            // SOCK-05: No focus side effects.
            let _ = resp_tx.send(ok(req_id, json!({"id": "main"})));
        }

        // -- debug.* --
        SocketCommand::DebugLayout { req_id, resp_tx } => {
            // SOCK-05: No focus side effects.
            let s = state.borrow();
            match s.split_engines.get(s.active_index) {
                Some(engine) => {
                    let data = engine.root.to_data();
                    let json_tree = serde_json::to_value(&data).unwrap_or(Value::Null);
                    let _ = resp_tx.send(ok(req_id, json!({"layout": json_tree})));
                }
                None => {
                    let _ = resp_tx.send(err(req_id, "no_workspace", "no active workspace"));
                }
            }
        }

        SocketCommand::DebugType { req_id, text, resp_tx } => {
            // SOCK-05: No focus side effects (sends text to active surface without changing focus).
            let s = state.borrow();
            if let Some(engine) = s.split_engines.get(s.active_index) {
                if let Some(pane_id) = engine.root.find_active_pane_id() {
                    if let Some(surface) = engine.root.find_surface_for_pane(pane_id) {
                        if !surface.is_null() {
                            let c_text = std::ffi::CString::new(text.clone()).unwrap_or_default();
                            unsafe {
                                crate::ghostty::ffi::ghostty_surface_text(
                                    surface,
                                    c_text.as_ptr(),
                                    c_text.to_bytes().len(),
                                );
                            }
                        }
                    }
                }
            }
            let _ = resp_tx.send(ok(req_id, json!({})));
        }

        // ── surface.* ────────────────────────────────────────────────────
        SocketCommand::SurfaceList { req_id, resp_tx } => {
            // SOCK-05: No focus side effects.
            let s = state.borrow();
            let mut panes: Vec<Value> = Vec::new();
            for (ws_idx, (ws, engine)) in s.workspaces.iter().zip(s.split_engines.iter()).enumerate() {
                for (pane_uuid, _pane_id, active) in engine.all_panes() {
                    panes.push(json!({
                        "uuid": pane_uuid.to_string(),
                        "workspace_uuid": ws.uuid.to_string(),
                        "active": active && ws_idx == s.active_index,
                    }));
                }
            }
            let _ = resp_tx.send(ok(req_id, json!({"surfaces": panes})));
        }

        SocketCommand::SurfaceSplit { req_id, id: _, direction, resp_tx } => {
            // Split the active pane in the active workspace.
            // SplitEngine::split_active splits by orientation and returns new pane_id.
            let orientation = if direction == "vertical" {
                gtk4::Orientation::Vertical
            } else {
                gtk4::Orientation::Horizontal
            };
            let result = {
                let mut s = state.borrow_mut();
                let idx = s.active_index;
                if let Some(engine) = s.split_engines.get_mut(idx) {
                    engine.split_active(orientation)
                        .and_then(|new_pane_id| {
                            // Find the uuid of the newly created pane.
                            engine.all_panes().into_iter()
                                .find(|(_, pid, _)| *pid == new_pane_id)
                                .map(|(uuid, _, _)| uuid.to_string())
                        })
                } else {
                    None
                }
            };
            match result {
                Some(uuid_str) => {
                    let _ = resp_tx.send(ok(req_id, json!({"uuid": uuid_str})));
                }
                None => {
                    let _ = resp_tx.send(err(req_id, "split_failed", "could not split pane"));
                }
            }
        }

        SocketCommand::SurfaceFocus { req_id, id, resp_tx } => {
            // SOCK-05: surface.focus IS a focus-intent command — allowed to change focus.
            let pane_id = {
                let s = state.borrow();
                s.split_engines.get(s.active_index)
                    .and_then(|engine| engine.find_pane_id_by_uuid(&id))
            };
            match pane_id {
                Some(pid) => {
                    let mut s = state.borrow_mut();
                    let idx = s.active_index;
                    if let Some(engine) = s.split_engines.get_mut(idx) {
                        engine.active_pane_id = pid;
                        engine.root.update_focus_css(pid);
                        engine.grab_active_focus();
                    }
                    drop(s);
                    let _ = resp_tx.send(ok(req_id, json!({})));
                }
                None => { let _ = resp_tx.send(err(req_id, "not_found", "surface not found")); }
            }
        }

        SocketCommand::SurfaceClose { req_id, id, resp_tx } => {
            // Close pane by uuid. Set it as active, then close_active().
            let pane_id = {
                let s = state.borrow();
                s.split_engines.get(s.active_index)
                    .and_then(|engine| engine.find_pane_id_by_uuid(&id))
            };
            match pane_id {
                Some(pid) => {
                    let result = {
                        let mut s = state.borrow_mut();
                        let idx = s.active_index;
                        if let Some(engine) = s.split_engines.get_mut(idx) {
                            engine.active_pane_id = pid;
                            engine.root.update_focus_css(pid);
                            engine.close_active()
                        } else {
                            None
                        }
                    };
                    match result {
                        Some(_) => { let _ = resp_tx.send(ok(req_id, json!({}))); }
                        None => { let _ = resp_tx.send(err(req_id, "close_failed", "cannot close last pane")); }
                    }
                }
                None => { let _ = resp_tx.send(err(req_id, "not_found", "surface not found")); }
            }
        }

        SocketCommand::SurfaceSendText { req_id, id, text, resp_tx } => {
            // SOCK-05: send_text is NOT a focus-intent command — NO focus change.
            let surface = {
                let s = state.borrow();
                if let Some(engine) = s.split_engines.get(s.active_index) {
                    if let Some(ref uuid_str) = id {
                        engine.find_surface_by_uuid(uuid_str)
                    } else {
                        engine.root.find_active_pane_id()
                            .and_then(|pid| engine.root.find_surface_for_pane(pid))
                    }
                } else { None }
            };
            if let Some(surf) = surface {
                if !surf.is_null() {
                    let c_text = std::ffi::CString::new(text.clone()).unwrap_or_default();
                    unsafe {
                        crate::ghostty::ffi::ghostty_surface_text(
                            surf,
                            c_text.as_ptr(),
                            c_text.to_bytes().len(),
                        );
                    }
                }
            }
            let _ = resp_tx.send(ok(req_id, json!({})));
        }

        SocketCommand::SurfaceSendKey { req_id, id, key, resp_tx } => {
            // SOCK-05: send_key is NOT a focus-intent command — NO focus change.
            // For Phase 3, single printable chars sent as text.
            // Complex key combos (ctrl+c, etc.) require ghostty_surface_key — Phase 4.
            let surface = {
                let s = state.borrow();
                if let Some(engine) = s.split_engines.get(s.active_index) {
                    if let Some(ref uuid_str) = id {
                        engine.find_surface_by_uuid(uuid_str)
                    } else {
                        engine.root.find_active_pane_id()
                            .and_then(|pid| engine.root.find_surface_for_pane(pid))
                    }
                } else { None }
            };
            if let Some(surf) = surface {
                if !surf.is_null() && key.len() == 1 {
                    let c_key = std::ffi::CString::new(key.clone()).unwrap_or_default();
                    unsafe {
                        crate::ghostty::ffi::ghostty_surface_text(
                            surf,
                            c_key.as_ptr(),
                            c_key.to_bytes().len(),
                        );
                    }
                }
            }
            let _ = resp_tx.send(ok(req_id, json!({})));
        }

        SocketCommand::SurfaceReadText { req_id, id: _, resp_tx } => {
            // SOCK-05: No focus side effects.
            // Stub — Ghostty screen buffer API not yet available. Phase 4.
            let _ = resp_tx.send(ok(req_id, json!({"text": ""})));
        }

        SocketCommand::SurfaceHealth { req_id, id, resp_tx } => {
            // SOCK-05: health is NOT focus-intent — NO focus change.
            let (found, has_attention) = {
                let s = state.borrow();
                if let Some(engine) = s.split_engines.get(s.active_index) {
                    if let Some(ref uuid_str) = id {
                        let alive = engine.find_surface_by_uuid(uuid_str).is_some();
                        let attn = engine.find_pane_id_by_uuid(uuid_str)
                            .map(|pid| engine.root.pane_has_attention(pid))
                            .unwrap_or(false);
                        (alive, attn)
                    } else {
                        let attn = engine.root.find_active_pane_id()
                            .map(|pid| engine.root.pane_has_attention(pid))
                            .unwrap_or(false);
                        (true, attn)
                    }
                } else { (false, false) }
            };
            let _ = resp_tx.send(ok(req_id, json!({"alive": found, "has_attention": has_attention})));
        }

        SocketCommand::SurfaceRefresh { req_id, id, resp_tx } => {
            // SOCK-05: refresh is NOT focus-intent — NO focus change.
            // Queue a render on the target surface's GLArea.
            let gl_area = {
                let s = state.borrow();
                if let Some(engine) = s.split_engines.get(s.active_index) {
                    let target_pane_id = if let Some(ref uuid_str) = id {
                        engine.find_pane_id_by_uuid(uuid_str)
                    } else {
                        engine.root.find_active_pane_id()
                    };
                    target_pane_id.and_then(|pid| engine.gl_area_for_pane(pid))
                } else { None }
            };
            if let Some(area) = gl_area {
                area.queue_render();
            }
            let _ = resp_tx.send(ok(req_id, json!({})));
        }

        // ── pane.* ───────────────────────────────────────────────────────────
        SocketCommand::PaneList { req_id, resp_tx } => {
            // SOCK-05: No focus side effects. Alias for surface.list.
            let s = state.borrow();
            let mut panes: Vec<Value> = Vec::new();
            for (ws_idx, (ws, engine)) in s.workspaces.iter().zip(s.split_engines.iter()).enumerate() {
                for (pane_uuid, _pane_id, active) in engine.all_panes() {
                    panes.push(json!({
                        "uuid": pane_uuid.to_string(),
                        "workspace_uuid": ws.uuid.to_string(),
                        "active": active && ws_idx == s.active_index,
                    }));
                }
            }
            let _ = resp_tx.send(ok(req_id, json!({"panes": panes})));
        }

        SocketCommand::PaneFocus { req_id, id, resp_tx } => {
            // SOCK-05: pane.focus IS focus-intent — allowed to change focus.
            let pane_id = {
                let s = state.borrow();
                if let Some(engine) = s.split_engines.get(s.active_index) {
                    id.as_ref().and_then(|uuid_str| engine.find_pane_id_by_uuid(uuid_str))
                } else { None }
            };
            match pane_id {
                Some(pid) => {
                    let mut s = state.borrow_mut();
                    let idx = s.active_index;
                    if let Some(engine) = s.split_engines.get_mut(idx) {
                        engine.active_pane_id = pid;
                        engine.root.update_focus_css(pid);
                        engine.grab_active_focus();
                    }
                    drop(s);
                    let _ = resp_tx.send(ok(req_id, json!({})));
                }
                None => { let _ = resp_tx.send(err(req_id, "not_found", "pane not found")); }
            }
        }

        SocketCommand::PaneLast { req_id, resp_tx } => {
            // SOCK-05: pane.last IS focus-intent — allowed to change focus.
            // Phase 3 stub: re-grab focus on current active pane. Phase 4 tracks focus history.
            {
                let s = state.borrow();
                if let Some(engine) = s.split_engines.get(s.active_index) {
                    engine.grab_active_focus();
                }
            }
            let _ = resp_tx.send(ok(req_id, json!({})));
        }

        // -- notification.* (Phase 4) --
        SocketCommand::NotificationList { req_id, resp_tx } => {
            // SOCK-05: No focus side effects. Read-only attention state query.
            let s = state.borrow();
            let notifications: Vec<Value> = s.workspaces.iter().map(|ws| {
                json!({
                    "workspace_uuid": ws.uuid.to_string(),
                    "workspace_name": ws.name,
                    "has_attention": ws.has_attention,
                })
            }).collect();
            let _ = resp_tx.send(ok(req_id, json!({"notifications": notifications})));
        }

        SocketCommand::NotificationClear { req_id, id, resp_tx } => {
            // SOCK-05: No focus side effects. Clears attention without switching workspace.
            let idx = {
                let s = state.borrow();
                s.workspaces.iter().position(|ws| ws.uuid.to_string() == id)
            };
            match idx {
                Some(i) => {
                    state.borrow_mut().clear_workspace_attention(i);
                    let _ = resp_tx.send(ok(req_id, json!({})));
                }
                None => {
                    let _ = resp_tx.send(err(req_id, "not_found", "workspace not found"));
                }
            }
        }

        // -- browser.* (Phase 8: D-04 lifecycle + streaming) --
        // SOCK-05: None of these commands steal focus.
        SocketCommand::BrowserOpen { req_id, url, workspace, resp_tx } => {
            let mut s = state.borrow_mut();
            // Lazy-init BrowserManager per D-05
            if s.browser_manager.is_none() {
                s.browser_manager = Some(crate::browser::BrowserManager::new());
            }
            let bm = s.browser_manager.as_mut().unwrap();
            // Ensure daemon is running (auto-start per D-05)
            if let Err(e) = bm.ensure_daemon() {
                let _ = resp_tx.send(err(req_id, "daemon_error", &e));
                return;
            }
            // Build params for agent-browser, including workspace if provided
            let mut open_params = serde_json::json!({"url": url});
            if let Some(ref ws) = workspace {
                open_params["workspace"] = serde_json::json!(ws);
            }
            match bm.send_command("navigate", open_params) {
                Ok(result) => {
                    // Allocate surface ref (D-06)
                    s.browser_surface_counter += 1;
                    let ref_id = s.browser_surface_counter;
                    let uuid = result.get("id")
                        .or_else(|| result.get("surface_id"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string();
                    s.browser_surface_refs.insert(ref_id, uuid.clone());
                    // Augment response with surface_ref
                    let mut response = result.clone();
                    if let Some(obj) = response.as_object_mut() {
                        obj.insert("surface_ref".to_string(), serde_json::json!(format!("surface:{}", ref_id)));
                        obj.insert("uuid".to_string(), serde_json::json!(uuid));
                    }
                    // Create preview pane in the active workspace split tree
                    if let Some(engine) = s.active_split_engine_mut() {
                        let _ = engine.split_active_with_preview();
                    }
                    let _ = resp_tx.send(ok(req_id, response));
                }
                Err(e) => {
                    let _ = resp_tx.send(err(req_id, "browser_error", &e));
                }
            }
        }

        SocketCommand::BrowserStreamEnable { req_id, resp_tx } => {
            let mut s = state.borrow_mut();
            if s.browser_manager.is_none() {
                s.browser_manager = Some(crate::browser::BrowserManager::new());
            }
            let bm = s.browser_manager.as_mut().unwrap();
            if let Err(e) = bm.ensure_daemon() {
                let _ = resp_tx.send(err(req_id, "daemon_error", &e));
                return;
            }
            match bm.send_command("stream_enable", serde_json::json!({})) {
                Ok(result) => {
                    // Find the Picture widget from the Preview pane in the active workspace.
                    // If no preview pane exists yet, create one first.
                    let picture = {
                        let engine = s.active_split_engine_mut();
                        if let Some(eng) = engine {
                            // Try to find existing Preview node's Picture
                            find_preview_picture(&eng.root)
                                .or_else(|| {
                                    // No preview pane yet -- create one
                                    eng.split_active_with_preview().map(|w| w.picture)
                                })
                        } else {
                            None
                        }
                    };

                    // Wire the WebSocket stream to the Picture widget (Gap 1 fix)
                    if let Some(pic) = picture {
                        let runtime = s.runtime_handle.clone();
                        let bm = s.browser_manager.as_mut().unwrap();
                        if let Some(ref rt) = runtime {
                            match bm.start_stream(rt, pic) {
                                Ok(()) => {
                                    // stream wired to preview pane
                                }
                                Err(e) => {
                                    eprintln!("cmux: stream enable failed: {}", e);
                                }
                            }
                        } else {
                            // no runtime handle
                        }
                    } else {
                        // no preview pane available
                    }

                    let _ = resp_tx.send(ok(req_id, result));
                }
                Err(e) => {
                    let _ = resp_tx.send(err(req_id, "stream_error", &e));
                }
            }
        }

        SocketCommand::BrowserStreamDisable { req_id, resp_tx } => {
            let mut s = state.borrow_mut();
            if let Some(ref mut bm) = s.browser_manager {
                match bm.send_command("stream_disable", serde_json::json!({})) {
                    Ok(result) => {
                        let _ = resp_tx.send(ok(req_id, result));
                    }
                    Err(e) => {
                        let _ = resp_tx.send(err(req_id, "stream_error", &e));
                    }
                }
            } else {
                let _ = resp_tx.send(err(req_id, "not_running", "No browser session active"));
            }
        }

        SocketCommand::BrowserList { req_id, resp_tx } => {
            let s = state.borrow();
            let surfaces: Vec<serde_json::Value> = s.browser_surface_refs.iter()
                .map(|(ref_id, uuid)| {
                    serde_json::json!({
                        "ref": format!("surface:{}", ref_id),
                        "uuid": uuid,
                        "status": "registered",
                    })
                })
                .collect();
            let _ = resp_tx.send(ok(req_id, serde_json::json!({"surfaces": surfaces})));
        }

        // -- browser.* generic proxy (P0/P1 parity) --
        SocketCommand::BrowserAction { req_id, action, mut params, surface_ref, resp_tx } => {
            let s = state.borrow();
            if let Some(ref bm) = s.browser_manager {
                // Resolve surface ref if provided
                if let Some(ref sref) = surface_ref {
                    match resolve_surface_ref(sref, &s.browser_surface_refs) {
                        Ok(uuid) => {
                            if let Some(obj) = params.as_object_mut() {
                                obj.remove("surface_ref");
                                obj.insert("surface_id".to_string(), serde_json::json!(uuid));
                            }
                        }
                        Err((msg, available)) => {
                            let _ = resp_tx.send(json!({
                                "id": req_id,
                                "ok": false,
                                "error": {"code": "surface_not_found", "message": msg},
                                "available": available,
                            }));
                            return;
                        }
                    }
                }
                // Translate cmux CLI action names to agent-browser action names
                let daemon_action = match action.as_str() {
                    "open" => "launch",
                    "goto" => "navigate",
                    "eval" => "evaluate",
                    "gethtml" => "innerhtml",
                    "stream.enable" => "stream_enable",
                    "stream.disable" => "stream_disable",
                    _ => &action,
                };
                match bm.send_command(daemon_action, params) {
                    Ok(result) => {
                        let _ = resp_tx.send(ok(req_id, result));
                    }
                    Err(e) => {
                        let _ = resp_tx.send(err(req_id, "browser_error", &e));
                    }
                }
            } else {
                let _ = resp_tx.send(err(req_id, "not_running", "No browser session active"));
            }
        }

        // -- Tier-2 stubs (D-10) --
        SocketCommand::NotImplemented { req_id, method, resp_tx } => {
            let _ = resp_tx.send(err(req_id, "not_implemented", &format!("{method} is not implemented")));
        }
    }
}

/// Walk the split tree to find the first Preview node's Picture widget.
fn find_preview_picture(node: &crate::split_engine::SplitNode) -> Option<gtk4::Picture> {
    match node {
        crate::split_engine::SplitNode::Preview { picture, .. } => Some(picture.clone()),
        crate::split_engine::SplitNode::Split { start, end, .. } => {
            find_preview_picture(start).or_else(|| find_preview_picture(end))
        }
        crate::split_engine::SplitNode::Leaf { .. } => None,
    }
}
