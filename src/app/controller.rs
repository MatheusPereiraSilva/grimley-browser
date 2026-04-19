use std::sync::Arc;

use tao::{
    event::Event,
    event_loop::{ControlFlow, EventLoop},
};

use crate::{
    browser::create_ui_webview,
    internal_pages::{render_internal_page, VisitedPages},
    pdf::create_pdf_fetcher,
    shield::create_shield_engine,
    storage::{
        create_app_storage,
        history::{load_history, save_history},
        session::{load_session, restore_tab_launches, save_session},
        settings::{load_settings, save_settings},
    },
    tabs::BrowserTabs,
    ui::create_window,
};

use super::{
    events::{handle_browser_action, handle_loaded_urls, handle_window_event},
    state::{create_loaded_urls, create_pdf_routes, create_pending_action},
    ui_sync::sync_ui,
};

pub(crate) fn run() {
    let event_loop = EventLoop::new();
    let window = create_window(&event_loop);
    let storage = create_app_storage().unwrap_or_else(|error| {
        panic!("Nao foi possivel inicializar o storage do Grimley: {error}");
    });
    let settings = load_settings(&storage).unwrap_or_else(|error| {
        tracing::warn!("Falha ao carregar settings persistidos: {error}");
        Default::default()
    });

    let pending_action = create_pending_action();
    let loaded_urls = create_loaded_urls();
    let pdf_routes = create_pdf_routes();
    let pdf_fetcher = create_pdf_fetcher();
    let shield_engine = create_shield_engine();
    shield_engine.set_observation_only(settings.shield.observation_only);
    shield_engine.set_rules_text(&settings.shield.custom_rules);

    let mut browser_tabs = if settings.restore_last_session {
        match load_session(&storage) {
            Ok(Some(snapshot)) => {
                let (active_index, launches) = restore_tab_launches(snapshot);
                BrowserTabs::restore(
                    &window,
                    Arc::clone(&loaded_urls),
                    Arc::clone(&pending_action),
                    Arc::clone(&pdf_routes),
                    Arc::clone(&pdf_fetcher),
                    Arc::clone(&shield_engine),
                    storage.clone(),
                    launches,
                    active_index,
                )
            }
            Ok(None) => BrowserTabs::new(
                &window,
                Arc::clone(&loaded_urls),
                Arc::clone(&pending_action),
                Arc::clone(&pdf_routes),
                Arc::clone(&pdf_fetcher),
                Arc::clone(&shield_engine),
                storage.clone(),
                &settings.home_page_url,
            ),
            Err(error) => {
                tracing::warn!("Falha ao carregar a sessao persistida: {error}");
                BrowserTabs::new(
                    &window,
                    Arc::clone(&loaded_urls),
                    Arc::clone(&pending_action),
                    Arc::clone(&pdf_routes),
                    Arc::clone(&pdf_fetcher),
                    Arc::clone(&shield_engine),
                    storage.clone(),
                    &settings.home_page_url,
                )
            }
        }
    } else {
        BrowserTabs::new(
            &window,
            Arc::clone(&loaded_urls),
            Arc::clone(&pending_action),
            Arc::clone(&pdf_routes),
            Arc::clone(&pdf_fetcher),
            Arc::clone(&shield_engine),
            storage.clone(),
            &settings.home_page_url,
        )
    };
    let ui_webview = create_ui_webview(&window, Arc::clone(&pending_action));
    let mut visited_pages = load_history(&storage).unwrap_or_else(|error| {
        tracing::warn!("Falha ao carregar o historico persistido: {error}");
        VisitedPages::default()
    });
    let mut last_ui_snapshot = None;

    if browser_tabs.active_tab().needs_render() {
        render_internal_page(
            browser_tabs.active_tab_mut(),
            &visited_pages,
            Arc::clone(&shield_engine),
            &storage,
            &window,
            Arc::clone(&loaded_urls),
            Arc::clone(&pending_action),
        );
    }

    sync_ui(&ui_webview, &browser_tabs, &mut last_ui_snapshot);

    if let Err(error) = save_settings(&storage, &settings) {
        tracing::warn!("Falha ao salvar settings iniciais: {error}");
    }
    if let Err(error) = save_history(&storage, &visited_pages) {
        tracing::warn!("Falha ao salvar historico inicial: {error}");
    }
    if let Err(error) = save_session(&storage, &browser_tabs) {
        tracing::warn!("Falha ao salvar sessao inicial: {error}");
    }

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        if let Some(action) = pending_action.lock().unwrap().take() {
            handle_browser_action(
                action,
                &mut browser_tabs,
                &window,
                &ui_webview,
                &loaded_urls,
                &pending_action,
                &pdf_routes,
                &pdf_fetcher,
                &shield_engine,
                &storage,
                &mut visited_pages,
                &mut last_ui_snapshot,
            );

            if let Err(error) = save_session(&storage, &browser_tabs) {
                tracing::warn!("Falha ao salvar a sessao das abas: {error}");
            }
        }

        let processed_loaded_urls = handle_loaded_urls(
            &mut browser_tabs,
            &window,
            &ui_webview,
            &loaded_urls,
            &pending_action,
            &mut visited_pages,
            &mut last_ui_snapshot,
            &shield_engine,
            &storage,
        );

        if processed_loaded_urls {
            if let Err(error) = save_history(&storage, &visited_pages) {
                tracing::warn!("Falha ao salvar o historico visitado: {error}");
            }
            if let Err(error) = save_session(&storage, &browser_tabs) {
                tracing::warn!("Falha ao salvar a sessao apos carregamento de URL: {error}");
            }
        }

        if let Event::WindowEvent { event, .. } = event {
            handle_window_event(&event, control_flow, &window, &ui_webview, &browser_tabs);
        }
    });
}
