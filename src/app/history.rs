use super::{
    constants::{HISTORY_PAGE_URL, NEW_TAB_PAGE_URL},
    escape::escape_html,
};

#[derive(Default)]
pub(crate) struct BrowserHistory {
    current: Option<String>,
    back_stack: Vec<String>,
    forward_stack: Vec<String>,
}

impl BrowserHistory {
    pub(crate) fn new(initial_url: &str) -> Self {
        Self {
            current: Some(initial_url.to_string()),
            back_stack: Vec::new(),
            forward_stack: Vec::new(),
        }
    }

    pub(crate) fn navigate_to(&mut self, url: &str) -> String {
        if self.current.as_deref() != Some(url) {
            if let Some(current) = self.current.take() {
                self.back_stack.push(current);
            }
            self.forward_stack.clear();
            self.current = Some(url.to_string());
        }

        url.to_string()
    }

    pub(crate) fn go_back(&mut self) -> Option<String> {
        let previous = self.back_stack.pop()?;

        if let Some(current) = self.current.take() {
            self.forward_stack.push(current);
        }

        self.current = Some(previous.clone());
        Some(previous)
    }

    pub(crate) fn go_forward(&mut self) -> Option<String> {
        let next = self.forward_stack.pop()?;

        if let Some(current) = self.current.take() {
            self.back_stack.push(current);
        }

        self.current = Some(next.clone());
        Some(next)
    }

    pub(crate) fn set_current(&mut self, url: &str) {
        self.current = Some(url.to_string());
    }

    pub(crate) fn can_go_back(&self) -> bool {
        !self.back_stack.is_empty()
    }

    pub(crate) fn can_go_forward(&self) -> bool {
        !self.forward_stack.is_empty()
    }
}

#[derive(Clone)]
struct HistoryEntry {
    url: String,
}

#[derive(Default)]
pub(crate) struct VisitedPages {
    entries: Vec<HistoryEntry>,
}

impl VisitedPages {
    pub(crate) fn record(&mut self, url: &str) {
        if url == HISTORY_PAGE_URL || url == NEW_TAB_PAGE_URL {
            return;
        }

        self.entries.push(HistoryEntry {
            url: url.to_string(),
        });

        if self.entries.len() > 250 {
            let overflow = self.entries.len() - 250;
            self.entries.drain(0..overflow);
        }
    }

    pub(crate) fn render_html(&self) -> String {
        let items = if self.entries.is_empty() {
            "<div class=\"empty\">Seu historico ainda esta vazio.</div>".to_string()
        } else {
            self.entries
                .iter()
                .rev()
                .enumerate()
                .map(|(index, entry)| {
                    format!(
                        "<a class=\"item\" href=\"{url}\"><span class=\"order\">{position}</span><span class=\"text\">{label}</span></a>",
                        url = escape_html(&entry.url),
                        position = self.entries.len() - index,
                        label = escape_html(&entry.url),
                    )
                })
                .collect::<Vec<_>>()
                .join("")
        };

        format!(
            r#"<!DOCTYPE html>
<html lang="pt-BR">
<head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>Historico</title>
    <style>
        :root {{
            color-scheme: light;
            --bg: #f6f8fb;
            --card: #ffffff;
            --border: #dfe3eb;
            --text: #1f2937;
            --muted: #6b7280;
            --accent: #1a73e8;
        }}
        * {{ box-sizing: border-box; }}
        body {{
            margin: 0;
            padding: 32px 20px 48px;
            font-family: "Segoe UI", Arial, sans-serif;
            color: var(--text);
            background:
                radial-gradient(circle at top left, rgba(26, 115, 232, 0.12), transparent 28%),
                var(--bg);
        }}
        .wrap {{ max-width: 900px; margin: 0 auto; }}
        .hero {{ margin-bottom: 24px; }}
        h1 {{ margin: 0 0 8px; font-size: 32px; }}
        p {{ margin: 0; color: var(--muted); font-size: 15px; }}
        .list {{ display: flex; flex-direction: column; gap: 10px; }}
        .item {{
            display: flex;
            align-items: center;
            gap: 14px;
            padding: 14px 16px;
            text-decoration: none;
            color: var(--text);
            background: var(--card);
            border: 1px solid var(--border);
            border-radius: 16px;
            transition: transform 0.16s ease, border-color 0.16s ease, box-shadow 0.16s ease;
        }}
        .item:hover {{
            transform: translateY(-1px);
            border-color: rgba(26, 115, 232, 0.35);
            box-shadow: 0 12px 30px rgba(15, 23, 42, 0.08);
        }}
        .order {{
            display: inline-flex;
            align-items: center;
            justify-content: center;
            width: 36px;
            height: 36px;
            border-radius: 999px;
            background: rgba(26, 115, 232, 0.12);
            color: var(--accent);
            font-weight: 700;
            flex: 0 0 auto;
        }}
        .text {{
            overflow: hidden;
            text-overflow: ellipsis;
            white-space: nowrap;
        }}
        .empty {{
            padding: 28px;
            text-align: center;
            color: var(--muted);
            background: var(--card);
            border: 1px dashed var(--border);
            border-radius: 16px;
        }}
    </style>
</head>
<body>
    <div class="wrap">
        <div class="hero">
            <h1>Historico</h1>
            <p>Selecione uma pagina visitada para abrir novamente.</p>
        </div>
        <div class="list">{items}</div>
    </div>
</body>
</html>"#,
            items = items
        )
    }
}
