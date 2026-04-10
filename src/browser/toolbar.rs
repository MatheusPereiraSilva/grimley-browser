use std::sync::{Arc, Mutex};

use tao::window::Window;
use wry::{http::Request, WebView, WebViewBuilder};

use super::{action::BrowserAction, layout::toolbar_bounds};

const TOOLBAR_HTML: &str = r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8" />
    <style>
        :root {
            color-scheme: light;
            --toolbar-bg: #f1f3f4;
            --tab-bg: #dfe3eb;
            --tab-active: #ffffff;
            --tab-hover: #e8eaed;
            --button-hover: #7ca5e2;
            --button-disabled: #bdc1c6;
            --input-bg: #ffffff;
            --input-border: #dfe1e5;
            --input-border-focus: #1a73e8;
            --text: #202124;
            --muted: #5f6368;
        }

        * {
            box-sizing: border-box;
        }

        html, body {
            width: 100%;
            height: 100%;
            overflow: hidden;
        }

        body {
            margin: 0;
            font-family: "Segoe UI", Arial, sans-serif;
            background: var(--toolbar-bg);
        }

        .toolbar-shell {
            width: 100%;
            height: 100%;
            padding: 8px 10px 10px;
            display: flex;
            flex-direction: column;
            gap: 8px;
        }

        .tabs-row {
            display: flex;
            align-items: center;
            gap: 8px;
            min-height: 30px;
        }

        .tabs-list {
            flex: 1;
            display: flex;
            align-items: center;
            gap: 6px;
            min-width: 0;
            overflow-x: auto;
            scrollbar-width: none;
        }

        .tabs-list::-webkit-scrollbar {
            display: none;
        }

        .tab,
        .icon-btn,
        .go-btn {
            border: 0;
            cursor: pointer;
            transition: background 0.16s ease, color 0.16s ease;
        }

        .tab {
            flex: 0 0 auto;
            max-width: 240px;
            padding: 0 6px 0 14px;
            height: 32px;
            display: flex;
            align-items: center;
            gap: 8px;
            border-radius: 14px 14px 0 0;
            background: var(--tab-bg);
            color: var(--muted);
            font-size: 13px;
        }

        .tab:hover {
            background: var(--tab-hover);
        }

        .tab.active {
            background: var(--tab-active);
            color: var(--text);
        }

        .tab-label {
            flex: 1;
            min-width: 0;
            white-space: nowrap;
            overflow: hidden;
            text-overflow: ellipsis;
            text-align: left;
        }

        .tab-close {
            width: 22px;
            height: 22px;
            flex: 0 0 auto;
            border: 0;
            border-radius: 999px;
            background: transparent;
            color: inherit;
            cursor: pointer;
            font-size: 14px;
            line-height: 1;
        }

        .tab-close:hover {
            background: rgba(95, 99, 104, 0.14);
        }

        .icon-btn,
        .go-btn {
            border-radius: 999px;
        }

        .icon-btn {
            width: 32px;
            height: 32px;
            background: transparent;
            color: var(--muted);
            font-size: 16px;
            line-height: 1;
        }

        .icon-btn:hover,
        .go-btn:hover {
            background: var(--button-hover);
        }

        .icon-btn:disabled {
            color: var(--button-disabled);
            cursor: default;
            background: transparent;
        }

        .address-row {
            display: flex;
            align-items: center;
            gap: 8px;
        }

        .address-wrap {
            flex: 1;
            display: flex;
            align-items: center;
            gap: 10px;
            min-width: 0;
            height: 40px;
            padding: 0 6px 0 14px;
            border-radius: 999px;
            background: var(--input-bg);
            border: 1px solid var(--input-border);
        }

        .address-wrap:focus-within {
            border-color: var(--input-border-focus);
            box-shadow: inset 0 0 0 1px var(--input-border-focus);
        }

        .site-dot {
            width: 14px;
            height: 14px;
            flex: 0 0 auto;
            border-radius: 999px;
            background: radial-gradient(circle at 35% 35%, #34a853, #1e8e3e);
        }

        #url {
            flex: 1;
            min-width: 0;
            border: 0;
            outline: none;
            background: transparent;
            color: var(--text);
            font-size: 14px;
        }

        #url::placeholder {
            color: var(--muted);
        }

        .go-btn {
            min-width: 44px;
            height: 32px;
            padding: 0 12px;
            color: #ffffff;
            font-size: 13px;
            font-weight: 600;
            background: #1a73e8;
        }
    </style>
</head>
<body>
    <div class="toolbar-shell">
        <div class="tabs-row">
            <div id="tabs" class="tabs-list"></div>
            <button class="icon-btn" onclick="newTab()" title="Nova aba">+</button>
        </div>

        <div class="address-row">
            <button id="back" class="icon-btn" onclick="back()" title="Voltar">&#8592;</button>
            <button id="forward" class="icon-btn" onclick="forward()" title="Avancar">&#8594;</button>
            <button class="icon-btn" onclick="openHistory()" title="Historico">&#9716;</button>

            <div class="address-wrap">
                <div class="site-dot"></div>
                <input id="url" placeholder="Digite uma URL" />
                <button class="go-btn" onclick="go()">Ir</button>
            </div>
        </div>
    </div>

    <script>
        const input = document.getElementById('url');
        const backButton = document.getElementById('back');
        const forwardButton = document.getElementById('forward');
        const tabsContainer = document.getElementById('tabs');

        function go() {
            window.ipc.postMessage('navigate:' + input.value);
        }

        function back() {
            window.ipc.postMessage('back');
        }

        function forward() {
            window.ipc.postMessage('forward');
        }

        function openHistory() {
            window.ipc.postMessage('show-history');
        }

        function newTab() {
            window.ipc.postMessage('new-tab');
        }

        function switchTab(index) {
            window.ipc.postMessage('switch-tab:' + index);
        }

        function closeTab(index) {
            window.ipc.postMessage('close-tab:' + index);
        }

        input.addEventListener('keydown', function(event) {
            if (event.key === 'Enter') go();
        });

        window.setUrl = function(url) {
            input.value = url;
        };

        window.setNavState = function(canGoBack, canGoForward) {
            backButton.disabled = !canGoBack;
            forwardButton.disabled = !canGoForward;
        };

        window.setTabs = function(tabs, activeIndex) {
            tabsContainer.innerHTML = '';

            tabs.forEach(function(tab, index) {
                const button = document.createElement('button');
                button.className = 'tab' + (index === activeIndex ? ' active' : '');
                button.title = tab.url;
                button.onclick = function() { switchTab(index); };

                const label = document.createElement('span');
                label.className = 'tab-label';
                label.textContent = tab.label;

                const close = document.createElement('button');
                close.className = 'tab-close';
                close.textContent = 'x';
                close.title = 'Fechar aba';
                close.onclick = function(event) {
                    event.stopPropagation();
                    closeTab(index);
                };

                button.appendChild(label);
                button.appendChild(close);
                tabsContainer.appendChild(button);
            });
        };
    </script>
</body>
</html>
"#;

pub fn create_ui_webview(
    window: &Window,
    pending_action: Arc<Mutex<Option<BrowserAction>>>,
) -> WebView {
    WebViewBuilder::new_as_child(window)
        .with_bounds(toolbar_bounds(window))
        .with_html(TOOLBAR_HTML)
        .with_ipc_handler(move |request: Request<String>| {
            if let Some(action) = parse_browser_action(request.body().trim()) {
                *pending_action.lock().unwrap() = Some(action);
            }
        })
        .build()
        .unwrap()
}

fn parse_browser_action(message: &str) -> Option<BrowserAction> {
    if message == "back" {
        Some(BrowserAction::Back)
    } else if message == "forward" {
        Some(BrowserAction::Forward)
    } else if message == "show-history" {
        Some(BrowserAction::ShowHistory)
    } else if message == "new-tab" {
        Some(BrowserAction::NewTab(None))
    } else if let Some(index) = message.strip_prefix("switch-tab:") {
        index.parse().ok().map(BrowserAction::SwitchTab)
    } else if let Some(index) = message.strip_prefix("close-tab:") {
        index.parse().ok().map(BrowserAction::CloseTab)
    } else if let Some(url) = message.strip_prefix("navigate:") {
        Some(BrowserAction::Navigate(url.to_string()))
    } else {
        None
    }
}
