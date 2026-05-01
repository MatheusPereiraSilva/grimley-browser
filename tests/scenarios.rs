use grimley_browser::testing::{
    export_document_with_derived_text, render_history_page, render_pdf_workspace_with_saved_note,
    BrowserHarness, HarnessTabKind, TestExportFormat,
};

#[test]
fn scenario_open_common_url() {
    let mut browser = BrowserHarness::new("https://grimley.dev");

    browser.navigate("example.com/docs");

    assert_eq!(browser.active_kind(), HarnessTabKind::Web);
    assert_eq!(browser.active_url(), "https://example.com/docs");
    assert_eq!(browser.history_len(), 1);
}

#[test]
fn scenario_open_pdf_url() {
    let mut browser = BrowserHarness::new("https://grimley.dev");

    browser.navigate("example.com/reference.pdf");
    let html = browser
        .render_active_page()
        .expect("Era esperado HTML para o workspace do PDF");

    assert_eq!(browser.active_kind(), HarnessTabKind::Pdf);
    assert!(html.contains("Anotar PDF original"));
    assert!(html.contains("Fluxos separados"));
    assert!(html.contains("Grifar selecao"));
    assert!(html.contains("Adicionar anotacao"));
    assert!(!html.contains("Desenho livre"));
    assert!(html.contains("grimley-pdf"));
}

#[test]
fn scenario_popup_opens_new_tab() {
    let mut browser = BrowserHarness::new("https://grimley.dev");

    browser.popup_new_tab("docs.rs");

    assert_eq!(browser.tab_count(), 2);
    assert_eq!(browser.active_kind(), HarnessTabKind::Web);
    assert_eq!(browser.active_url(), "https://docs.rs");
}

#[test]
fn scenario_history_is_rendered() {
    let mut browser = BrowserHarness::new("https://grimley.dev");

    browser.navigate("https://example.com");
    browser.navigate("https://docs.rs");
    browser.open_history();

    let html = browser
        .render_active_page()
        .expect("Era esperado HTML da pagina de historico");

    assert!(html.contains("<title>Historico</title>"));
    assert!(html.contains("https://example.com"));
    assert!(html.contains("https://docs.rs"));

    let direct_html = render_history_page(&["https://example.com", "https://docs.rs"]);
    assert!(direct_html.contains("Selecione uma pagina visitada"));
}

#[test]
fn scenario_pdf_annotation_is_embedded_in_workspace() {
    let html =
        render_pdf_workspace_with_saved_note("https://example.com/report.pdf", "Nota de teste")
            .expect("Era esperado renderizar o workspace com anotacoes");

    assert!(html.contains("Anotar PDF original"));
    assert!(html.contains("Nota de teste"));
    assert!(html.contains("Adicionar anotacao"));
    assert!(html.contains("save-pdf-annotations"));
}

#[test]
fn scenario_exports_derived_document() {
    let plain_text = export_document_with_derived_text(
        "https://example.com/report.pdf",
        "Resumo final do documento",
        TestExportFormat::PlainText,
    )
    .expect("Era esperado exportar o texto derivado");
    let project = export_document_with_derived_text(
        "https://example.com/report.pdf",
        "Resumo final do documento",
        TestExportFormat::Project,
    )
    .expect("Era esperado exportar o projeto Grimley");

    assert_eq!(plain_text.filename, "report.txt");
    assert_eq!(plain_text.body, "Resumo final do documento");
    assert_eq!(project.filename, "report.grimley.json");
    assert!(project
        .body
        .contains("\"derivedText\":\"Resumo final do documento\""));
}
