use crate::shield::{diagnostics::ShieldRequestLog, ShieldEngineHandle};

use super::InternalPageRenderer;

pub(crate) struct ShieldPage {
    engine: ShieldEngineHandle,
}

impl ShieldPage {
    pub(crate) fn new(engine: ShieldEngineHandle) -> Self {
        Self { engine }
    }
}

impl InternalPageRenderer for ShieldPage {
    fn render(&self) -> String {
        let snapshot = self.engine.diagnostics_snapshot();
        let status = if self.engine.observation_only() {
            "Observation Only"
        } else {
            "Enforcing"
        };
        let requests = if snapshot.requests.is_empty() {
            "<div class=\"empty\">Nenhuma requisicao observada ainda. Navegue um pouco e volte para acompanhar o Grimley Shield.</div>".to_string()
        } else {
            snapshot
                .requests
                .iter()
                .rev()
                .map(render_request_row)
                .collect::<Vec<_>>()
                .join("")
        };

        format!(
            r#"<!DOCTYPE html>
<html lang="pt-BR">
<head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>Grimley Shield</title>
    <style>
        :root {{
            color-scheme: light;
            --bg: #eef3ed;
            --panel: rgba(255, 255, 255, 0.86);
            --panel-strong: #ffffff;
            --border: rgba(37, 70, 54, 0.14);
            --text: #132219;
            --muted: #597061;
            --accent: #1f6f4a;
            --accent-soft: rgba(31, 111, 74, 0.12);
            --warn: #986200;
            --danger: #8f2b2b;
            --shadow: 0 22px 60px rgba(19, 34, 25, 0.1);
        }}
        * {{ box-sizing: border-box; }}
        body {{
            margin: 0;
            min-height: 100vh;
            color: var(--text);
            font-family: Georgia, "Times New Roman", serif;
            background:
                radial-gradient(circle at top left, rgba(31, 111, 74, 0.18), transparent 26%),
                radial-gradient(circle at bottom right, rgba(195, 145, 68, 0.14), transparent 22%),
                linear-gradient(180deg, #f6fbf5 0%, var(--bg) 100%);
        }}
        .shell {{
            max-width: 1180px;
            margin: 0 auto;
            padding: 36px 20px 60px;
        }}
        .hero {{
            display: grid;
            grid-template-columns: minmax(0, 1.3fr) minmax(280px, 0.7fr);
            gap: 20px;
            align-items: stretch;
            margin-bottom: 22px;
        }}
        .hero-card,
        .status-card,
        .metric,
        .requests-card {{
            border: 1px solid var(--border);
            border-radius: 28px;
            background: var(--panel);
            backdrop-filter: blur(14px);
            box-shadow: var(--shadow);
        }}
        .hero-card {{
            padding: 28px;
        }}
        .eyebrow {{
            display: inline-flex;
            align-items: center;
            gap: 8px;
            padding: 8px 12px;
            border-radius: 999px;
            background: var(--accent-soft);
            color: var(--accent);
            font-size: 12px;
            letter-spacing: 0.12em;
            text-transform: uppercase;
        }}
        h1 {{
            margin: 16px 0 10px;
            font-size: clamp(36px, 5vw, 56px);
            line-height: 0.96;
            letter-spacing: -0.04em;
        }}
        .hero p,
        .status-copy,
        .subtle {{
            margin: 0;
            color: var(--muted);
            font-family: "Segoe UI", Arial, sans-serif;
            line-height: 1.55;
        }}
        .status-card {{
            padding: 24px;
            display: flex;
            flex-direction: column;
            justify-content: space-between;
            gap: 18px;
        }}
        .status-badge {{
            display: inline-flex;
            align-items: center;
            width: fit-content;
            padding: 7px 12px;
            border-radius: 999px;
            background: rgba(195, 145, 68, 0.14);
            color: var(--warn);
            font-family: "Segoe UI", Arial, sans-serif;
            font-size: 12px;
            font-weight: 700;
            text-transform: uppercase;
            letter-spacing: 0.08em;
        }}
        .status-value {{
            font-size: 30px;
            font-weight: 700;
            letter-spacing: -0.04em;
        }}
        .metrics {{
            display: grid;
            grid-template-columns: repeat(4, minmax(0, 1fr));
            gap: 14px;
            margin-bottom: 22px;
        }}
        .metric {{
            padding: 20px;
        }}
        .metric-label {{
            margin-bottom: 10px;
            color: var(--muted);
            font-family: "Segoe UI", Arial, sans-serif;
            font-size: 12px;
            font-weight: 700;
            letter-spacing: 0.08em;
            text-transform: uppercase;
        }}
        .metric-value {{
            font-size: 34px;
            font-weight: 700;
            letter-spacing: -0.05em;
        }}
        .requests-card {{
            padding: 24px;
        }}
        .requests-head {{
            display: flex;
            align-items: end;
            justify-content: space-between;
            gap: 16px;
            margin-bottom: 18px;
        }}
        .requests-title {{
            margin: 0;
            font-size: 28px;
            letter-spacing: -0.04em;
        }}
        .request-list {{
            display: flex;
            flex-direction: column;
            gap: 12px;
        }}
        .request {{
            padding: 18px;
            border: 1px solid var(--border);
            border-radius: 22px;
            background: var(--panel-strong);
        }}
        .request-top {{
            display: flex;
            align-items: center;
            justify-content: space-between;
            gap: 12px;
            margin-bottom: 12px;
        }}
        .request-url {{
            min-width: 0;
            font-family: "Segoe UI", Arial, sans-serif;
            font-size: 15px;
            font-weight: 600;
            word-break: break-word;
        }}
        .request-seq {{
            flex: 0 0 auto;
            padding: 6px 10px;
            border-radius: 999px;
            background: var(--accent-soft);
            color: var(--accent);
            font-family: "Segoe UI", Arial, sans-serif;
            font-size: 12px;
            font-weight: 700;
        }}
        .chips {{
            display: flex;
            flex-wrap: wrap;
            gap: 8px;
            margin-bottom: 12px;
        }}
        .chip {{
            display: inline-flex;
            align-items: center;
            padding: 6px 10px;
            border-radius: 999px;
            background: #f1f6f0;
            border: 1px solid var(--border);
            color: var(--text);
            font-family: "Segoe UI", Arial, sans-serif;
            font-size: 12px;
            font-weight: 600;
        }}
        .chip.warn {{
            background: rgba(195, 145, 68, 0.12);
            color: var(--warn);
        }}
        .chip.block {{
            background: rgba(143, 43, 43, 0.1);
            color: var(--danger);
        }}
        .details {{
            display: grid;
            grid-template-columns: repeat(2, minmax(0, 1fr));
            gap: 10px 18px;
            font-family: "Segoe UI", Arial, sans-serif;
            font-size: 13px;
            color: var(--muted);
        }}
        .details strong {{
            color: var(--text);
        }}
        .muted {{
            color: var(--muted);
        }}
        .empty {{
            padding: 34px 20px;
            border: 1px dashed var(--border);
            border-radius: 20px;
            text-align: center;
            color: var(--muted);
            font-family: "Segoe UI", Arial, sans-serif;
            background: rgba(255, 255, 255, 0.72);
        }}
        @media (max-width: 900px) {{
            .hero,
            .metrics,
            .details {{
                grid-template-columns: 1fr;
            }}
        }}
    </style>
</head>
<body>
    <main class="shell">
        <section class="hero">
            <article class="hero-card">
                <div class="eyebrow">Grimley Shield</div>
                <h1>Telemetria de requests, sem travar sua navegação.</h1>
                <p>O Shield ja observa URLs, classifica o tipo de recurso e registra o que seria bloqueado pelas regras ativas. Nesta etapa ele ainda opera sem bloquear nada de verdade.</p>
            </article>
            <aside class="status-card">
                <div>
                    <div class="status-badge">{status}</div>
                    <div class="status-value">{rule_count} regra(s) carregada(s)</div>
                </div>
                <p class="status-copy">Modo atual: o engine calcula a decisao e escreve o diagnostico, mas devolve <strong>Allow</strong> para manter a compatibilidade enquanto o pipeline amadurece.</p>
            </aside>
        </section>

        <section class="metrics">
            <article class="metric">
                <div class="metric-label">Requests vistos</div>
                <div class="metric-value">{seen_count}</div>
            </article>
            <article class="metric">
                <div class="metric-label">Bloqueados agora</div>
                <div class="metric-value">{blocked_count}</div>
            </article>
            <article class="metric">
                <div class="metric-label">Bloquearia</div>
                <div class="metric-value">{would_block_count}</div>
            </article>
            <article class="metric">
                <div class="metric-label">Excecoes aplicadas</div>
                <div class="metric-value">{exception_count}</div>
            </article>
        </section>

        <section class="requests-card">
            <div class="requests-head">
                <div>
                    <h2 class="requests-title">Requests observados</h2>
                    <p class="subtle">Entradas mais recentes primeiro. Navegue para popular a trilha do Shield e use este painel como base para os proximos passos de bloqueio real.</p>
                </div>
            </div>
            <div class="request-list">{requests}</div>
        </section>
    </main>
</body>
</html>"#,
            status = escape_html(status),
            rule_count = self.engine.rule_count(),
            seen_count = snapshot.seen_count,
            blocked_count = snapshot.blocked_count,
            would_block_count = snapshot.would_block_count,
            exception_count = snapshot.exception_count,
            requests = requests,
        )
    }
}

fn render_request_row(request: &ShieldRequestLog) -> String {
    let matched_rules = if request.matched_rules.is_empty() {
        "<span class=\"muted\">Nenhuma regra casou.</span>".to_string()
    } else {
        request
            .matched_rules
            .iter()
            .map(|rule| format!("<span class=\"chip block\">{}</span>", escape_html(rule)))
            .collect::<Vec<_>>()
            .join("")
    };
    let exception = request
        .applied_exception
        .as_deref()
        .map(escape_html)
        .unwrap_or_else(|| "Nenhuma".to_string());
    let host = request
        .host
        .as_deref()
        .map(escape_html)
        .unwrap_or_else(|| "Host indisponivel".to_string());
    let source = request
        .source_url
        .as_deref()
        .map(escape_html)
        .unwrap_or_else(|| "Origem nao capturada".to_string());

    format!(
        r#"<article class="request">
    <div class="request-top">
        <div class="request-url">{url}</div>
        <div class="request-seq">#{sequence}</div>
    </div>
    <div class="chips">
        <span class="chip">{context}</span>
        <span class="chip">{resource_type}</span>
        <span class="chip warn">Aplicado: {applied_action}</span>
        <span class="chip block">Proposto: {proposed_action}</span>
    </div>
    <div class="details">
        <div><strong>Host:</strong> {host}</div>
        <div><strong>Aba:</strong> {tab_id}</div>
        <div><strong>Origem:</strong> {source}</div>
        <div><strong>Excecao:</strong> {exception}</div>
        <div><strong>Momento:</strong> {observed_at_ms}</div>
        <div><strong>Regras:</strong> {matched_rules}</div>
    </div>
</article>"#,
        url = escape_html(&request.url),
        sequence = request.sequence,
        context = escape_html(request.context.label()),
        resource_type = escape_html(request.resource_type.label()),
        applied_action = escape_html(&request.applied_action),
        proposed_action = escape_html(&request.proposed_action),
        host = host,
        tab_id = request.tab_id,
        source = source,
        exception = exception,
        observed_at_ms = request.observed_at_ms,
        matched_rules = matched_rules,
    )
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
