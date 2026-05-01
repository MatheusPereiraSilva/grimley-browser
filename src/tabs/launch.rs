use crate::{
    browser::{is_pdf_url, normalize_url},
    internal_pages::internal_page_kind_for_url,
};

use super::TabLaunch;

pub(crate) fn launch_for_requested_url(url: Option<&str>) -> TabLaunch {
    match url {
        Some(value) => {
            let normalized = normalize_url(value);
            if let Some(kind) = internal_page_kind_for_url(&normalized) {
                TabLaunch::internal(kind)
            } else if is_pdf_url(&normalized) {
                TabLaunch::pdf(normalized)
            } else {
                TabLaunch::regular(normalized)
            }
        }
        None => TabLaunch::new_tab(),
    }
}

pub(crate) fn launch_for_pdf_url(pdf_url: &str) -> TabLaunch {
    TabLaunch::pdf(normalize_url(pdf_url))
}

#[cfg(test)]
mod tests {
    use crate::internal_pages::InternalPageKind;

    use super::*;

    #[test]
    fn selects_new_tab_when_url_is_missing() {
        let launch = launch_for_requested_url(None);

        assert!(matches!(launch, TabLaunch::Internal(_)));
        match launch {
            TabLaunch::Internal(document) => {
                assert_eq!(document.kind(), InternalPageKind::NewTab);
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn selects_internal_tab_for_internal_url() {
        let launch = launch_for_requested_url(Some("grimley://historico"));

        match launch {
            TabLaunch::Internal(document) => {
                assert_eq!(document.kind(), InternalPageKind::History);
            }
            _ => panic!("Era esperado um launch interno"),
        }
    }

    #[test]
    fn selects_pdf_tab_for_pdf_url() {
        let launch = launch_for_requested_url(Some("example.com/report.pdf"));

        match launch {
            TabLaunch::Pdf { origin_url, .. } => {
                assert_eq!(origin_url, "https://example.com/report.pdf");
            }
            _ => panic!("Era esperado um launch de PDF"),
        }
    }

    #[test]
    fn selects_web_tab_for_regular_url() {
        let launch = launch_for_requested_url(Some("example.com"));

        match launch {
            TabLaunch::Web { url } => {
                assert_eq!(url, "https://example.com");
            }
            _ => panic!("Era esperado um launch web"),
        }
    }
}
