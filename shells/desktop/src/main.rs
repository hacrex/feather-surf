// FeatherSurf Desktop Browser Shell
//
// Minimal browser using tao (windowing) + wry (webview).
// Uses OS native webview (WebView2 on Windows, WebKit on macOS/Linux).

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tao::event::{Event, StartCause, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tao::keyboard::KeyCode;
use tao::window::{WindowBuilder, WindowId};
use wry::webview::{WebView, WebViewBuilder};

struct Tab {
    title: String,
    url: String,
    _webview: WebView,
}

struct BrowserState {
    tabs: Vec<Tab>,
    active_tab: usize,
    next_tab_id: u64,
}

const START_PAGE_HTML: &str = r#"<!DOCTYPE html>
<html>
<head>
<style>
  * { margin: 0; padding: 0; box-sizing: border-box; }
  body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; background: #1a1a2e; color: #e0e0e0; height: 100vh; display: flex; flex-direction: column; }
  .chrome { background: #16213e; padding: 6px 12px; display: flex; align-items: center; gap: 8px; border-bottom: 1px solid #0f3460; }
  .nav-btn { background: #0f3460; border: none; color: #e0e0e0; padding: 6px 12px; border-radius: 4px; cursor: pointer; font-size: 14px; }
  .nav-btn:hover { background: #533483; }
  .address-bar { flex: 1; background: #1a1a2e; border: 1px solid #0f3460; color: #e0e0e0; padding: 6px 12px; border-radius: 4px; font-size: 14px; outline: none; }
  .address-bar:focus { border-color: #533483; }
  .content { flex: 1; display: flex; align-items: center; justify-content: center; flex-direction: column; padding: 40px; }
  h1 { color: #e94560; font-size: 48px; margin-bottom: 16px; }
  p { color: #a0a0b0; font-size: 16px; margin-bottom: 8px; }
  .search-box { width: 500px; max-width: 80vw; margin-top: 24px; }
  .search-box input { width: 100%; padding: 12px 16px; font-size: 16px; background: #16213e; border: 2px solid #0f3460; color: #e0e0e0; border-radius: 8px; outline: none; }
  .search-box input:focus { border-color: #e94560; }
  .shortcuts { display: flex; gap: 16px; margin-top: 32px; flex-wrap: wrap; justify-content: center; }
  .shortcut { background: #16213e; border: 1px solid #0f3460; padding: 16px 24px; border-radius: 8px; cursor: pointer; text-decoration: none; color: #e0e0e0; transition: border-color 0.2s; }
  .shortcut:hover { border-color: #e94560; }
  .shortcut-name { font-weight: bold; font-size: 14px; }
  .shortcut-url { font-size: 12px; color: #a0a0b0; margin-top: 4px; }
</style>
</head>
<body>
  <div class="chrome">
    <button class="nav-btn" onclick="goBack()" title="Back">&#9664;</button>
    <button class="nav-btn" onclick="goForward()" title="Forward">&#9654;</button>
    <button class="nav-btn" onclick="reload()" title="Reload">&#8635;</button>
    <input class="address-bar" id="url" placeholder="Enter URL or search..." autofocus
           onkeydown="if(event.key==='Enter')navigate(this.value)">
  </div>
  <div class="content">
    <h1>FeatherSurf</h1>
    <p>Lightweight, privacy-first browser</p>
    <div class="search-box">
      <input id="search" placeholder="Search the web or enter URL..."
             onkeydown="if(event.key==='Enter')navigate(this.value)">
    </div>
    <div class="shortcuts">
      <div class="shortcut" onclick="navigate('https://github.com')">
        <div class="shortcut-name">GitHub</div>
        <div class="shortcut-url">github.com</div>
      </div>
      <div class="shortcut" onclick="navigate('https://wikipedia.org')">
        <div class="shortcut-name">Wikipedia</div>
        <div class="shortcut-url">wikipedia.org</div>
      </div>
      <div class="shortcut" onclick="navigate('https://news.ycombinator.com')">
        <div class="shortcut-name">Hacker News</div>
        <div class="shortcut-url">news.ycombinator.com</div>
      </div>
      <div class="shortcut" onclick="navigate('https://rust-lang.org')">
        <div class="shortcut-name">Rust</div>
        <div class="shortcut-url">rust-lang.org</div>
      </div>
    </div>
  </div>
  <script>
    function navigate(input) {
      let url = input.trim();
      if (!url) return;
      if (!url.startsWith('http://') && !url.startsWith('https://')) {
        if (url.includes('.') && !url.includes(' ')) {
          url = 'https://' + url;
        } else {
          url = 'https://www.google.com/search?q=' + encodeURIComponent(url);
        }
      }
      window.ipc.postMessage(JSON.stringify({ cmd: 'navigate', url: url }));
    }
    function goBack() { window.ipc.postMessage(JSON.stringify({ cmd: 'back' })); }
    function goForward() { window.ipc.postMessage(JSON.stringify({ cmd: 'forward' })); }
    function reload() { window.ipc.postMessage(JSON.stringify({ cmd: 'reload' })); }
    window.ipc.onMessage = function(msg) {
      const data = JSON.parse(msg);
      if (data.cmd === 'update_url') {
        document.getElementById('url').value = data.url;
      }
    };
    document.getElementById('url').focus();
  </script>
</body>
</html>"#;

fn normalize_url(input: &str) -> String {
    let url = input.trim();
    if url.is_empty() {
        return String::new();
    }
    if url.starts_with("http://") || url.starts_with("https://") {
        url.to_string()
    } else if url.contains('.') && !url.contains(' ') {
        format!("https://{}", url)
    } else {
        // Simple URL encoding for search query
        let encoded: String = url
            .bytes()
            .map(|b| match b {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    (b as char).to_string()
                }
                b' ' => "+".to_string(),
                _ => format!("%{:02X}", b),
            })
            .collect();
        format!("https://www.google.com/search?q={}", encoded)
    }
}

fn main() {
    let event_loop = EventLoopBuilder::new().build();

    let window = WindowBuilder::new()
        .with_title("FeatherSurf")
        .with_inner_size(tao::dpi::LogicalSize::new(1280.0, 800.0))
        .with_min_inner_size(tao::dpi::LogicalSize::new(800.0, 600.0))
        .build(&event_loop)
        .unwrap();

    let window_id = window.id();

    let webview = WebViewBuilder::new()
        .unwrap()
        .with_html(START_PAGE_HTML)
        .with_devtools(true)
        .with_ipc_handler({
            let window = Arc::new(Mutex::new(Some(window)));
            move |webview, msg| {
                if let Ok(cmd) = serde_json::from_str::<serde_json::Value>(&msg) {
                    match cmd["cmd"].as_str() {
                        Some("navigate") => {
                            if let Some(url) = cmd["url"].as_str() {
                                let normalized = normalize_url(url);
                                let _ = webview.navigate(&normalized);
                                // Update address bar
                                let js = format!(
                                    "window.ipc.onMessage(JSON.stringify({{ cmd: 'update_url', url: '{}' }}));",
                                    normalized.replace('\\', "\\\\").replace('\'', "\\'")
                                );
                                let _ = webview.evaluate_javascript(&js);
                            }
                        }
                        Some("back") => {
                            let _ = webview.go_back();
                        }
                        Some("forward") => {
                            let _ = webview.go_forward();
                        }
                        Some("reload") => {
                            let _ = webview.reload();
                        }
                        _ => {}
                    }
                }
            }
        })
        .build(&window)
        .unwrap();

    // Store webview (must keep it alive)
    let _webview = Box::new(webview);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::NewEvents(StartCause::Poll) => {}
            Event::WindowEvent { event, window_id: id, .. } if id == window_id => {
                match event {
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    WindowEvent::KeyboardInput { event: key_event, .. } => {
                        if key_event.state == tao::event::ElementState::Pressed {
                            if let tao::keyboard::KeyCode::F12 = key_event.physical_key {
                                // DevTools toggle could go here
                            }
                        }
                    }
                    WindowEvent::Resized(size) => {
                        let _ = _webview.set_bounds(wry::webview::Rect {
                            x: 0,
                            y: 0,
                            width: size.width,
                            height: size.height,
                        });
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    });
}
