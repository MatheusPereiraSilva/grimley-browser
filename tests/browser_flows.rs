use grimley_browser::testing::{BrowserHarness, HarnessTabKind};

#[test]
fn opens_new_tab_and_makes_it_active() {
    let mut browser = BrowserHarness::new("https://grimley.dev");

    let new_index = browser.open_new_tab(None);

    assert_eq!(new_index, 1);
    assert_eq!(browser.tab_count(), 2);
    assert_eq!(browser.active_index(), 1);
    assert_eq!(browser.active_kind(), HarnessTabKind::Internal);
    assert_eq!(browser.active_url(), "grimley://nova-aba");
}

#[test]
fn supports_back_and_forward_navigation() {
    let mut browser = BrowserHarness::new("https://grimley.dev");

    browser.navigate("example.com");
    browser.navigate("docs.rs");

    assert_eq!(browser.back().as_deref(), Some("https://example.com"));
    assert_eq!(browser.active_url(), "https://example.com");
    assert_eq!(browser.forward().as_deref(), Some("https://docs.rs"));
    assert_eq!(browser.active_url(), "https://docs.rs");
}

#[test]
fn opens_pdf_in_workspace() {
    let mut browser = BrowserHarness::new("https://grimley.dev");

    browser.navigate("https://example.com/report.pdf");
    let html = browser
        .render_active_page()
        .expect("Era esperado HTML do workspace de PDF");

    assert_eq!(browser.active_kind(), HarnessTabKind::Pdf);
    assert_eq!(browser.active_url(), "https://example.com/report.pdf");
    assert!(html.contains("Exportar anotacoes"));
    assert!(html.contains("Grifar selecao"));
    assert!(html.contains("Adicionar anotacao"));
    assert!(!html.contains("Desenho livre"));
}

#[test]
fn closes_active_tab_and_selects_remaining_neighbor() {
    let mut browser = BrowserHarness::new("https://grimley.dev");

    browser.open_new_tab(Some("https://example.com"));
    browser.open_new_tab(Some("https://docs.rs"));
    browser.close_active_tab();

    assert_eq!(browser.tab_count(), 2);
    assert_eq!(browser.active_index(), 1);
    assert_eq!(browser.active_url(), "https://example.com");
}

#[test]
fn switches_between_tabs() {
    let mut browser = BrowserHarness::new("https://grimley.dev");

    browser.open_new_tab(Some("https://example.com"));
    browser.open_new_tab(Some("https://docs.rs"));

    assert!(browser.switch_tab(1));
    assert_eq!(browser.active_url(), "https://example.com");
    assert_eq!(browser.active_kind(), HarnessTabKind::Web);

    assert!(browser.switch_tab(0));
    assert_eq!(browser.active_url(), "https://grimley.dev");
}
