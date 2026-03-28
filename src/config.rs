use gtk4::gdk::{Key, ModifierType};
use std::collections::HashMap;
use std::path::PathBuf;

/// Top-level config loaded from ~/.config/cmux/config.toml.
/// Phase 5: shortcuts section only (D-07).
#[derive(serde::Deserialize, Default, Debug)]
pub struct Config {
    #[serde(default)]
    pub shortcuts: ShortcutConfig,
    #[serde(default)]
    pub ui: UiConfig,
}

/// Per-action shortcut overrides. Each value is a GTK accelerator string (e.g. "<Ctrl>n").
/// None means "use default".
#[derive(serde::Deserialize, Default, Debug)]
pub struct ShortcutConfig {
    pub new_workspace: Option<String>,
    pub close_workspace: Option<String>,
    pub next_workspace: Option<String>,
    pub prev_workspace: Option<String>,
    pub rename_workspace: Option<String>,
    pub toggle_sidebar: Option<String>,
    pub split_right: Option<String>,
    pub split_down: Option<String>,
    pub close_pane: Option<String>,
    pub new_ssh_workspace: Option<String>,
    pub focus_left: Option<String>,
    pub focus_right: Option<String>,
    pub focus_up: Option<String>,
    pub focus_down: Option<String>,
    pub workspace_1: Option<String>,
    pub workspace_2: Option<String>,
    pub workspace_3: Option<String>,
    pub workspace_4: Option<String>,
    pub workspace_5: Option<String>,
    pub workspace_6: Option<String>,
    pub workspace_7: Option<String>,
    pub workspace_8: Option<String>,
    pub workspace_9: Option<String>,
    pub browser_open: Option<String>,
    pub browser_close: Option<String>,
}

/// UI configuration section -- [ui] in config.toml (D-16).
#[derive(serde::Deserialize, Default, Debug)]
pub struct UiConfig {
    #[serde(default)]
    pub header_bar: HeaderBarConfig,
}

/// Header bar configuration -- [ui.header_bar] in config.toml (D-16).
/// Requires app restart to take effect.
#[derive(serde::Deserialize, Debug)]
pub struct HeaderBarConfig {
    /// "gtk" (default, full header bar), "custom" (user-specified buttons), "none" (no header bar)
    #[serde(default = "default_header_style")]
    pub style: String,
    /// Button names for left side (only used when style="custom")
    pub buttons_left: Option<Vec<String>>,
    /// Button names for right side (only used when style="custom")
    pub buttons_right: Option<Vec<String>>,
}

fn default_header_style() -> String {
    "gtk".to_string()
}

impl Default for HeaderBarConfig {
    fn default() -> Self {
        Self {
            style: default_header_style(),
            buttons_left: None,
            buttons_right: None,
        }
    }
}

/// All bindable shortcut actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShortcutAction {
    NewWorkspace,
    CloseWorkspace,
    NextWorkspace,
    PrevWorkspace,
    RenameWorkspace,
    ToggleSidebar,
    SplitRight,
    SplitDown,
    ClosePane,
    NewSshWorkspace,
    FocusLeft,
    FocusRight,
    FocusUp,
    FocusDown,
    Workspace1,
    Workspace2,
    Workspace3,
    Workspace4,
    Workspace5,
    Workspace6,
    Workspace7,
    Workspace8,
    Workspace9,
    BrowserOpen,
    BrowserClose,
}

/// HashMap-based shortcut lookup table built from config + defaults.
pub struct ShortcutMap {
    map: HashMap<(ModifierType, Key), ShortcutAction>,
}

/// Known shortcut action names for unknown-key detection.
const KNOWN_SHORTCUTS: &[&str] = &[
    "new_workspace", "close_workspace", "next_workspace", "prev_workspace",
    "rename_workspace", "toggle_sidebar", "split_right", "split_down",
    "close_pane", "new_ssh_workspace", "focus_left", "focus_right", "focus_up", "focus_down",
    "workspace_1", "workspace_2", "workspace_3", "workspace_4",
    "workspace_5", "workspace_6", "workspace_7", "workspace_8", "workspace_9",
    "browser_open", "browser_close",
];

/// Modifier mask for lookup: ignore Caps Lock, Num Lock, etc.
const MOD_MASK: ModifierType = ModifierType::from_bits_truncate(
    ModifierType::CONTROL_MASK.bits()
        | ModifierType::SHIFT_MASK.bits()
        | ModifierType::ALT_MASK.bits(),
);

/// Returns the config file path.
/// Respects $XDG_CONFIG_HOME/cmux/config.toml; falls back to ~/.config/cmux/config.toml (CFG-04).
pub fn config_path() -> PathBuf {
    let base = std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        format!("{home}/.config")
    });
    PathBuf::from(base).join("cmux").join("config.toml")
}

/// Load config from disk. Always returns a usable Config (D-10).
/// Missing file is silent; read/parse errors warn to stderr and fall back to defaults.
pub fn load_config() -> Config {
    let path = config_path();
    let content = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Config::default();
        }
        Err(e) => {
            eprintln!("cmux: config read error at {}: {e}", path.display());
            return Config::default();
        }
    };

    warn_unknown_shortcuts(&content);

    match toml::from_str::<Config>(&content) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("cmux: config parse error at {}: {e}", path.display());
            Config::default()
        }
    }
}

/// Warn about unknown keys in the [shortcuts] table (D-03).
fn warn_unknown_shortcuts(content: &str) {
    let table: toml::Value = match content.parse() {
        Ok(v) => v,
        Err(_) => return, // Parse errors are reported by load_config
    };
    if let Some(shortcuts) = table.get("shortcuts").and_then(|v| v.as_table()) {
        for key in shortcuts.keys() {
            if !KNOWN_SHORTCUTS.contains(&key.as_str()) {
                eprintln!("cmux: unknown shortcut action '{}' in config, ignoring", key);
            }
        }
    }
}

impl ShortcutMap {
    /// Build lookup table from config, falling back to defaults for unset/invalid entries.
    pub fn from_config(config: &ShortcutConfig) -> Self {
        let entries: &[(ShortcutAction, &Option<String>, &str)] = &[
            (ShortcutAction::NewWorkspace,    &config.new_workspace,    "<Ctrl>n"),
            (ShortcutAction::CloseWorkspace,  &config.close_workspace,  "<Ctrl><Shift>w"),
            (ShortcutAction::NextWorkspace,   &config.next_workspace,   "<Ctrl>bracketright"),
            (ShortcutAction::PrevWorkspace,   &config.prev_workspace,   "<Ctrl>bracketleft"),
            (ShortcutAction::RenameWorkspace, &config.rename_workspace, "<Ctrl><Shift>r"),
            (ShortcutAction::ToggleSidebar,   &config.toggle_sidebar,   "<Ctrl>b"),
            (ShortcutAction::SplitRight,      &config.split_right,      "<Ctrl>d"),
            (ShortcutAction::SplitDown,       &config.split_down,       "<Ctrl><Shift>d"),
            (ShortcutAction::ClosePane,       &config.close_pane,       "<Ctrl><Shift>x"),
            (ShortcutAction::NewSshWorkspace, &config.new_ssh_workspace, "<Ctrl><Shift>s"),
            (ShortcutAction::FocusLeft,       &config.focus_left,       "<Ctrl><Shift>Left"),
            (ShortcutAction::FocusRight,      &config.focus_right,      "<Ctrl><Shift>Right"),
            (ShortcutAction::FocusUp,         &config.focus_up,         "<Ctrl><Shift>Up"),
            (ShortcutAction::FocusDown,       &config.focus_down,       "<Ctrl><Shift>Down"),
            (ShortcutAction::Workspace1,      &config.workspace_1,      "<Ctrl>1"),
            (ShortcutAction::Workspace2,      &config.workspace_2,      "<Ctrl>2"),
            (ShortcutAction::Workspace3,      &config.workspace_3,      "<Ctrl>3"),
            (ShortcutAction::Workspace4,      &config.workspace_4,      "<Ctrl>4"),
            (ShortcutAction::Workspace5,      &config.workspace_5,      "<Ctrl>5"),
            (ShortcutAction::Workspace6,      &config.workspace_6,      "<Ctrl>6"),
            (ShortcutAction::Workspace7,      &config.workspace_7,      "<Ctrl>7"),
            (ShortcutAction::Workspace8,      &config.workspace_8,      "<Ctrl>8"),
            (ShortcutAction::Workspace9,      &config.workspace_9,      "<Ctrl>9"),
            (ShortcutAction::BrowserOpen,     &config.browser_open,     "<Ctrl><Shift>b"),
            (ShortcutAction::BrowserClose,    &config.browser_close,    "<Ctrl><Shift>q"),
        ];

        let mut map = HashMap::new();

        for (action, config_val, default_accel) in entries {
            let accel_str = config_val.as_deref().unwrap_or(*default_accel);
            let action_name = format!("{:?}", action);

            if let Some((key, mods)) = gtk4::accelerator_parse(accel_str) {
                map.insert((mods & MOD_MASK, key), *action);
            } else {
                // D-11: invalid accelerator — warn and use default
                eprintln!(
                    "cmux: invalid shortcut '{}' for {}, using default '{}'",
                    accel_str, action_name, default_accel
                );
                if let Some((key, mods)) = gtk4::accelerator_parse(*default_accel) {
                    map.insert((mods & MOD_MASK, key), *action);
                }
            }
        }

        ShortcutMap { map }
    }

    /// Look up a shortcut action for the given modifier+key combination.
    /// Masks modifiers to ignore Caps Lock, Num Lock, etc.
    /// Normalizes keyval to lowercase because GTK4 key events give uppercase
    /// when Shift is held (e.g. Key::R), but accelerator_parse stores lowercase
    /// with the Shift modifier flag (e.g. Key::r + SHIFT_MASK).
    pub fn lookup(&self, mods: ModifierType, key: Key) -> Option<ShortcutAction> {
        let masked = mods & MOD_MASK;
        let lower_key = key.to_lower();
        self.map.get(&(masked, lower_key)).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_path_xdg() {
        // Temporarily set XDG_CONFIG_HOME and verify config_path() uses it.
        std::env::set_var("XDG_CONFIG_HOME", "/tmp/test-xdg-config");
        let path = config_path();
        assert_eq!(path, PathBuf::from("/tmp/test-xdg-config/cmux/config.toml"));
        std::env::remove_var("XDG_CONFIG_HOME");
    }

    #[test]
    fn test_load_config_missing_file() {
        // Point to a nonexistent dir so load_config returns defaults silently.
        std::env::set_var("XDG_CONFIG_HOME", "/tmp/cmux-test-nonexistent-dir-xyz");
        let config = load_config();
        assert!(config.shortcuts.new_workspace.is_none());
        assert!(config.shortcuts.close_workspace.is_none());
        std::env::remove_var("XDG_CONFIG_HOME");
    }

    #[test]
    fn test_load_config_empty_file() {
        let dir = std::env::temp_dir().join(format!("cmux-cfg-empty-{}", std::process::id()));
        let cfg_dir = dir.join("cmux");
        std::fs::create_dir_all(&cfg_dir).unwrap();
        let cfg_file = cfg_dir.join("config.toml");
        std::fs::write(&cfg_file, "").unwrap();

        std::env::set_var("XDG_CONFIG_HOME", dir.to_str().unwrap());
        let config = load_config();
        assert!(config.shortcuts.new_workspace.is_none());
        std::env::remove_var("XDG_CONFIG_HOME");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_load_config_valid_shortcuts() {
        let dir = std::env::temp_dir().join(format!("cmux-cfg-valid-{}", std::process::id()));
        let cfg_dir = dir.join("cmux");
        std::fs::create_dir_all(&cfg_dir).unwrap();
        let cfg_file = cfg_dir.join("config.toml");
        std::fs::write(&cfg_file, "[shortcuts]\nnew_workspace = \"<Ctrl>t\"\n").unwrap();

        std::env::set_var("XDG_CONFIG_HOME", dir.to_str().unwrap());
        let config = load_config();
        assert_eq!(config.shortcuts.new_workspace, Some("<Ctrl>t".to_string()));
        std::env::remove_var("XDG_CONFIG_HOME");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_load_config_invalid_toml() {
        let dir = std::env::temp_dir().join(format!("cmux-cfg-invalid-{}", std::process::id()));
        let cfg_dir = dir.join("cmux");
        std::fs::create_dir_all(&cfg_dir).unwrap();
        let cfg_file = cfg_dir.join("config.toml");
        std::fs::write(&cfg_file, "[shortcuts\n").unwrap();

        std::env::set_var("XDG_CONFIG_HOME", dir.to_str().unwrap());
        let config = load_config();
        // Falls back to defaults on parse error
        assert!(config.shortcuts.new_workspace.is_none());
        std::env::remove_var("XDG_CONFIG_HOME");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_ui_config_default() {
        let config: Config = toml::from_str("").unwrap();
        assert_eq!(config.ui.header_bar.style, "gtk");
        assert!(config.ui.header_bar.buttons_left.is_none());
    }

    #[test]
    fn test_ui_config_custom_style() {
        let config: Config = toml::from_str(r#"
[ui.header_bar]
style = "none"
buttons_left = ["new_workspace"]
buttons_right = ["split_right", "toggle_sidebar"]
"#).unwrap();
        assert_eq!(config.ui.header_bar.style, "none");
        assert_eq!(config.ui.header_bar.buttons_left.as_ref().unwrap().len(), 1);
        assert_eq!(config.ui.header_bar.buttons_right.as_ref().unwrap().len(), 2);
    }

    // Tests that require GTK4 initialization (accelerator_parse).
    // These will only work in environments with a display (or virtual display).

    #[test]
    fn test_shortcut_map_defaults() {
        if gtk4::init().is_err() {
            eprintln!("Skipping test_shortcut_map_defaults: GTK4 init failed (headless)");
            return;
        }
        let smap = ShortcutMap::from_config(&ShortcutConfig::default());
        // Ctrl+N should map to NewWorkspace
        let result = smap.lookup(ModifierType::CONTROL_MASK, Key::n);
        assert_eq!(result, Some(ShortcutAction::NewWorkspace));
    }

    #[test]
    fn test_shortcut_map_custom() {
        if gtk4::init().is_err() {
            eprintln!("Skipping test_shortcut_map_custom: GTK4 init failed (headless)");
            return;
        }
        let config = ShortcutConfig {
            new_workspace: Some("<Ctrl>t".to_string()),
            ..Default::default()
        };
        let smap = ShortcutMap::from_config(&config);
        // Ctrl+T should now map to NewWorkspace
        assert_eq!(smap.lookup(ModifierType::CONTROL_MASK, Key::t), Some(ShortcutAction::NewWorkspace));
        // Ctrl+N should no longer map to NewWorkspace
        assert_eq!(smap.lookup(ModifierType::CONTROL_MASK, Key::n), None);
    }
}
