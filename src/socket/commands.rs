use serde_json::Value;

pub type RespTx = tokio::sync::oneshot::Sender<Value>;

/// Commands dispatched from tokio accept loop to GTK main thread.
/// All variants carry `req_id` (echoed in response) and `resp_tx` (result channel).
/// GTK/AppState reads and mutations happen ONLY in handlers.rs on the main thread.
#[allow(dead_code)]
pub enum SocketCommand {
    // -- system.* --
    Ping             { req_id: Value, resp_tx: RespTx },
    Identify         { req_id: Value, resp_tx: RespTx },
    Capabilities     { req_id: Value, resp_tx: RespTx },

    // -- workspace.* --
    WorkspaceList    { req_id: Value, resp_tx: RespTx },
    WorkspaceCurrent { req_id: Value, resp_tx: RespTx },
    WorkspaceCreate  { req_id: Value, remote_target: Option<String>, resp_tx: RespTx },
    /// `id` is the workspace UUID string from the client.
    WorkspaceSelect  { req_id: Value, id: String, resp_tx: RespTx },
    WorkspaceClose   { req_id: Value, id: String, resp_tx: RespTx },
    WorkspaceRename  { req_id: Value, id: String, name: String, resp_tx: RespTx },
    WorkspaceNext    { req_id: Value, resp_tx: RespTx },
    WorkspacePrev    { req_id: Value, resp_tx: RespTx },
    WorkspaceLast    { req_id: Value, resp_tx: RespTx },
    WorkspaceReorder { req_id: Value, id: String, position: usize, resp_tx: RespTx },

    // -- surface.* (implemented in Plan 04) --
    SurfaceList      { req_id: Value, resp_tx: RespTx },
    SurfaceSplit     { req_id: Value, id: Option<String>, direction: String, resp_tx: RespTx },
    SurfaceFocus     { req_id: Value, id: String, resp_tx: RespTx },
    SurfaceClose     { req_id: Value, id: String, resp_tx: RespTx },
    SurfaceSendText  { req_id: Value, id: Option<String>, text: String, resp_tx: RespTx },
    SurfaceSendKey   { req_id: Value, id: Option<String>, key: String, resp_tx: RespTx },
    SurfaceReadText  { req_id: Value, id: Option<String>, resp_tx: RespTx },
    SurfaceHealth    { req_id: Value, id: Option<String>, resp_tx: RespTx },
    SurfaceRefresh   { req_id: Value, id: Option<String>, resp_tx: RespTx },

    // -- pane.* (implemented in Plan 04) --
    PaneList         { req_id: Value, resp_tx: RespTx },
    PaneFocus        { req_id: Value, id: Option<String>, resp_tx: RespTx },
    PaneLast         { req_id: Value, resp_tx: RespTx },

    // -- window.* --
    WindowList       { req_id: Value, resp_tx: RespTx },
    WindowCurrent    { req_id: Value, resp_tx: RespTx },

    // -- debug.* --
    DebugLayout      { req_id: Value, resp_tx: RespTx },
    DebugType        { req_id: Value, text: String, resp_tx: RespTx },

    // -- notification.* (Phase 4: NOTF-01, NOTF-02) --
    NotificationList   { req_id: Value, resp_tx: RespTx },
    NotificationClear  { req_id: Value, id: String, resp_tx: RespTx },

    // -- browser.* (Phase 8: D-04 lifecycle + streaming) --
    BrowserOpen          { req_id: Value, url: String, workspace: Option<String>, resp_tx: RespTx },
    BrowserStreamEnable  { req_id: Value, resp_tx: RespTx },
    BrowserStreamDisable { req_id: Value, resp_tx: RespTx },
    BrowserList          { req_id: Value, resp_tx: RespTx },

    // -- browser.* generic proxy (P0/P1 parity) --
    BrowserAction    { req_id: Value, action: String, params: Value, surface_ref: Option<String>, resp_tx: RespTx },

    // -- Tier-2 stub (not_implemented) --
    NotImplemented   { req_id: Value, method: String, resp_tx: RespTx },
}
