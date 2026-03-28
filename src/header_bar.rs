use gtk4::prelude::*;

/// Build the header bar per D-04, D-05, D-06.
/// Returns None if config style is "none".
pub fn build_header_bar(config: &crate::config::Config) -> Option<gtk4::HeaderBar> {
    if config.ui.header_bar.style == "none" {
        return None;
    }

    let header = gtk4::HeaderBar::new();
    header.add_css_class("cmux-headerbar");

    // Left side: workspace actions (D-06)

    // [+] New Workspace (D-05)
    let new_ws_btn = gtk4::Button::from_icon_name("tab-new-symbolic");
    new_ws_btn.set_tooltip_text(Some("New Workspace (Ctrl+N)"));
    new_ws_btn.set_action_name(Some("win.new-workspace"));
    new_ws_btn.add_css_class("headerbar-btn");
    header.pack_start(&new_ws_btn);

    // [Browser] New Browser (D-05, D-07)
    let browser_btn = gtk4::Button::from_icon_name("web-browser-symbolic");
    browser_btn.set_tooltip_text(Some("New Browser (Ctrl+Shift+B)"));
    browser_btn.set_action_name(Some("win.browser-open"));
    browser_btn.add_css_class("headerbar-btn");
    header.pack_start(&browser_btn);

    // Right side: pane/view actions (D-06)
    // Note: pack_end adds right-to-left, so order is reversed from visual

    // Hamburger menu (D-11) — rightmost, so pack_end first
    let menu_model = crate::menus::build_hamburger_menu();
    let menu_btn = gtk4::MenuButton::new();
    menu_btn.set_icon_name("open-menu-symbolic");
    menu_btn.set_tooltip_text(Some("Menu"));
    menu_btn.set_menu_model(Some(&menu_model));
    menu_btn.add_css_class("headerbar-btn");
    header.pack_end(&menu_btn);

    // Toggle Sidebar (D-05) — second from right
    let sidebar_btn = gtk4::Button::from_icon_name("sidebar-show-symbolic");
    sidebar_btn.set_tooltip_text(Some("Toggle Sidebar (Ctrl+B)"));
    sidebar_btn.set_action_name(Some("win.toggle-sidebar"));
    sidebar_btn.add_css_class("headerbar-btn");
    header.pack_end(&sidebar_btn);

    // Split Vertical / Split Down (D-05)
    let split_v_btn = gtk4::Button::from_icon_name("object-flip-vertical-symbolic");
    split_v_btn.set_tooltip_text(Some("Split Down (Ctrl+Shift+D)"));
    split_v_btn.set_action_name(Some("win.split-down"));
    split_v_btn.add_css_class("headerbar-btn");
    header.pack_end(&split_v_btn);

    // Split Horizontal / Split Right (D-05)
    let split_h_btn = gtk4::Button::from_icon_name("view-dual-symbolic");
    split_h_btn.set_tooltip_text(Some("Split Right (Ctrl+D)"));
    split_h_btn.set_action_name(Some("win.split-right"));
    split_h_btn.add_css_class("headerbar-btn");
    header.pack_end(&split_h_btn);

    Some(header)
}
