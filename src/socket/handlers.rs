// src/socket/handlers.rs — GTK main thread command dispatch

use crate::socket::commands::SocketCommand;
use serde_json::{json, Value};

/// Build a success response with the given result payload.
fn ok(req_id: Value, result: Value) -> Value {
    json!({"id": req_id, "ok": true, "result": result})
}

/// Build an error response.
fn err(req_id: Value, code: &str, message: &str) -> Value {
    json!({"id": req_id, "ok": false, "error": {"code": code, "message": message}})
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
            let methods = [
                "system.ping", "system.identify", "system.capabilities",
                "workspace.list", "workspace.current", "workspace.create",
                "workspace.select", "workspace.close", "workspace.rename",
                "workspace.next", "workspace.previous", "workspace.last", "workspace.reorder",
                "surface.list", "surface.split", "surface.focus", "surface.close",
                "surface.send_text", "surface.send_key", "surface.read_text",
                "surface.health", "surface.refresh",
                "pane.list", "pane.focus", "pane.last",
                "window.list", "window.current",
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
                    "uuid": ws.uuid.to_string(),
                    "name": ws.name,
                    "active": i == s.active_index,
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

        SocketCommand::WorkspaceCreate { req_id, resp_tx } => {
            // SOCK-05: create_workspace calls switch_to_index internally (existing behavior).
            let id = state.borrow_mut().create_workspace();
            let s = state.borrow();
            let uuid_str = s.workspaces.iter()
                .find(|ws| ws.id == id)
                .map(|ws| ws.uuid.to_string())
                .unwrap_or_default();
            let _ = resp_tx.send(ok(req_id, json!({"uuid": uuid_str})));
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

        // -- surface.* and pane.* -- implemented in Plan 04
        SocketCommand::SurfaceList { req_id, resp_tx } |
        SocketCommand::SurfaceSplit { req_id, resp_tx, .. } |
        SocketCommand::SurfaceFocus { req_id, resp_tx, .. } |
        SocketCommand::SurfaceClose { req_id, resp_tx, .. } |
        SocketCommand::SurfaceSendText { req_id, resp_tx, .. } |
        SocketCommand::SurfaceSendKey { req_id, resp_tx, .. } |
        SocketCommand::SurfaceReadText { req_id, resp_tx, .. } |
        SocketCommand::SurfaceHealth { req_id, resp_tx, .. } |
        SocketCommand::SurfaceRefresh { req_id, resp_tx, .. } |
        SocketCommand::PaneList { req_id, resp_tx } |
        SocketCommand::PaneFocus { req_id, resp_tx, .. } |
        SocketCommand::PaneLast { req_id, resp_tx } => {
            let _ = resp_tx.send(err(req_id, "not_implemented", "planned for Plan 04"));
        }

        // -- Tier-2 stubs (D-10) --
        SocketCommand::NotImplemented { req_id, method, resp_tx } => {
            let _ = resp_tx.send(err(req_id, "not_implemented", &format!("{method} is not implemented")));
        }
    }
}
