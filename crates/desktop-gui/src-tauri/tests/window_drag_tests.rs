use serde_json::Value;
use std::fs;
use std::path::PathBuf;

fn get_project_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // crates/desktop-gui/src-tauri -> root
    manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .expect("Failed to locate project root")
        .to_path_buf()
}

#[test]
fn test_tauri_window_overlay_config() {
    let root = get_project_root();
    let conf_path = root.join("crates/desktop-gui/src-tauri/tauri.conf.json");
    assert!(conf_path.exists(), "tauri.conf.json not found at {:?}", conf_path);

    let content = fs::read_to_string(&conf_path).expect("Failed to read tauri.conf.json");
    let json: Value = serde_json::from_str(&content).expect("Failed to parse tauri.conf.json");

    let windows = json
        .get("app")
        .and_then(|a| a.get("windows"))
        .and_then(|w| w.as_array())
        .expect("Missing app.windows array");

    let main_window = windows
        .iter()
        .find(|w| w.get("label").and_then(|l| l.as_str()) == Some("main"))
        .expect("Missing 'main' window in tauri.conf.json");

    assert_eq!(
        main_window.get("titleBarStyle").and_then(|v| v.as_str()),
        Some("Overlay"),
        "Main window must have titleBarStyle: 'Overlay' for native macOS seamless header"
    );

    assert_eq!(
        main_window.get("hiddenTitle").and_then(|v| v.as_bool()),
        Some(true),
        "Main window must have hiddenTitle: true to prevent default titlebar from overlaying custom header"
    );

    let min_width = main_window.get("minWidth").and_then(|v| v.as_u64()).unwrap_or(0);
    assert!(min_width >= 800, "Window minWidth should be >= 800 (got {})", min_width);

    let min_height = main_window.get("minHeight").and_then(|v| v.as_u64()).unwrap_or(0);
    assert!(min_height >= 600, "Window minHeight should be >= 600 (got {})", min_height);
}

#[test]
fn test_tauri_window_drag_capability_permissions() {
    let root = get_project_root();
    let cap_path = root.join("crates/desktop-gui/src-tauri/capabilities/default.json");
    assert!(cap_path.exists(), "capabilities/default.json not found at {:?}", cap_path);

    let content = fs::read_to_string(&cap_path).expect("Failed to read capabilities/default.json");
    let json: Value = serde_json::from_str(&content).expect("Failed to parse default.json");

    let windows = json
        .get("windows")
        .and_then(|w| w.as_array())
        .expect("Missing windows array in default.json");

    let targets_main = windows
        .iter()
        .any(|w| w.as_str() == Some("main") || w.as_str() == Some("*"));
    assert!(targets_main, "capabilities/default.json must target 'main' window");

    let permissions = json
        .get("permissions")
        .and_then(|p| p.as_array())
        .expect("Missing permissions array in default.json");

    let perm_strings: Vec<&str> = permissions.iter().filter_map(|p| p.as_str()).collect();

    assert!(
        perm_strings.contains(&"core:window:allow-start-dragging"),
        "Tauri v2 requires 'core:window:allow-start-dragging' for data-tauri-drag-region or startDragging() IPC"
    );

    assert!(
        perm_strings.contains(&"core:window:allow-start-resize-dragging"),
        "Missing 'core:window:allow-start-resize-dragging' capability"
    );

    assert!(
        perm_strings.contains(&"core:window:allow-toggle-maximize"),
        "Missing 'core:window:allow-toggle-maximize' for titlebar double-click support"
    );

    assert!(
        perm_strings.contains(&"core:window:default"),
        "Missing 'core:window:default' base window permissions"
    );
}

#[test]
fn test_html_drag_regions_isolated_from_interactive_controls() {
    let root = get_project_root();
    let html_path = root.join("crates/desktop-gui/ui/index.html");
    assert!(html_path.exists(), "index.html not found at {:?}", html_path);

    let html = fs::read_to_string(&html_path).expect("Failed to read index.html");

    // 1. Parent header should NOT have data-tauri-drag-region to avoid ancestor matching on child buttons
    assert!(
        !html.contains("<header class=\"app-header\" data-tauri-drag-region>"),
        "Parent <header class=\"app-header\"> must NOT have data-tauri-drag-region"
    );
    assert!(
        !html.contains("<header data-tauri-drag-region class=\"app-header\">"),
        "Parent header must NOT have data-tauri-drag-region"
    );

    // 2. Dedicated drag regions must have data-tauri-drag-region
    assert!(
        html.contains("class=\"header-traffic-lights-spacer\" data-tauri-drag-region")
            || html.contains("data-tauri-drag-region class=\"header-traffic-lights-spacer\""),
        ".header-traffic-lights-spacer must have data-tauri-drag-region"
    );

    assert!(
        html.contains("class=\"header-drag-space\" data-tauri-drag-region")
            || html.contains("data-tauri-drag-region class=\"header-drag-space\""),
        ".header-drag-space must have data-tauri-drag-region"
    );

    assert!(
        html.contains("class=\"header-drag-space-end\" data-tauri-drag-region")
            || html.contains("data-tauri-drag-region class=\"header-drag-space-end\""),
        ".header-drag-space-end must have data-tauri-drag-region"
    );

    // 3. Interactive buttons must NOT have data-tauri-drag-region
    let button_lines: Vec<&str> = html
        .lines()
        .filter(|line| line.contains("<button"))
        .collect();

    for btn in button_lines {
        assert!(
            !btn.contains("data-tauri-drag-region"),
            "Button elements must NOT have data-tauri-drag-region: {}",
            btn
        );
    }
}

#[test]
fn test_css_drag_region_rules() {
    let root = get_project_root();
    let css_path = root.join("crates/desktop-gui/ui/style.css");
    assert!(css_path.exists(), "style.css not found at {:?}", css_path);

    let css = fs::read_to_string(&css_path).expect("Failed to read style.css");

    // 1. data-tauri-drag-region must disable user-select
    assert!(
        css.contains("[data-tauri-drag-region]") && css.contains("user-select: none"),
        "[data-tauri-drag-region] must set user-select: none to avoid text selection during dragging"
    );

    // 2. Non-standard -webkit-app-region must not be used on the header
    assert!(
        !css.contains("-webkit-app-region: drag"),
        "style.css should not use invalid/conflicting -webkit-app-region: drag"
    );

    // 3. Traffic lights spacer width >= 70px
    let spacer_block = css
        .split(".header-traffic-lights-spacer")
        .nth(1)
        .and_then(|s| s.split('}').next())
        .expect("Missing .header-traffic-lights-spacer CSS block");

    assert!(
        spacer_block.contains("width: 80px") || spacer_block.contains("width: 70px"),
        "Traffic light spacer must have width >= 70px (found: {})",
        spacer_block
    );

    // 4. Drag space flex expands
    let drag_space_block = css
        .split(".header-drag-space")
        .nth(1)
        .and_then(|s| s.split('}').next())
        .expect("Missing .header-drag-space CSS block");

    assert!(
        drag_space_block.contains("flex: 1"),
        ".header-drag-space must have flex: 1 to fill available header space"
    );
}

#[test]
fn test_js_window_dragging_logic() {
    let root = get_project_root();
    let js_path = root.join("crates/desktop-gui/ui/app.js");
    assert!(js_path.exists(), "app.js not found at {:?}", js_path);

    let js = fs::read_to_string(&js_path).expect("Failed to read app.js");

    assert!(
        js.contains("function setupWindowDragging()"),
        "app.js must define setupWindowDragging()"
    );

    // Ensure we don't have conflicting mousedown startDragging call
    assert!(
        !js.contains("document.addEventListener('mousedown'"),
        "app.js must not add redundant mousedown listener that collides with Tauri's native data-tauri-drag-region"
    );

    // Ensure dblclick toggles maximize on drag regions
    assert!(
        js.contains("document.addEventListener('dblclick'"),
        "app.js should listen for dblclick to support macOS titlebar maximize"
    );

    assert!(
        js.contains("toggleMaximize()"),
        "app.js dblclick handler should call toggleMaximize()"
    );
}

#[test]
fn test_rust_command_registration() {
    let root = get_project_root();
    let lib_path = root.join("crates/desktop-gui/src-tauri/src/lib.rs");
    assert!(lib_path.exists(), "lib.rs not found at {:?}", lib_path);

    let lib = fs::read_to_string(&lib_path).expect("Failed to read lib.rs");

    assert!(
        lib.contains("fn start_dragging("),
        "lib.rs must declare start_dragging command"
    );

    assert!(
        lib.contains("start_dragging"),
        "generate_handler! in lib.rs must include start_dragging"
    );
}
