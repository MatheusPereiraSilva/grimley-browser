pub(crate) const HOME_PAGE_URL: &str = "https://www.google.com";

pub(crate) fn normalize_url(raw_url: &str) -> String {
    let trimmed = raw_url.trim();

    if trimmed.is_empty() {
        return "https://www.google.com".to_string();
    }

    if trimmed.contains("://") {
        trimmed.to_string()
    } else {
        format!("https://{}", trimmed)
    }
}

pub(crate) fn escape_js_string(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

pub(crate) fn is_pdf_url(url: &str) -> bool {
    let lower = url.trim().to_ascii_lowercase();
    let without_fragment = lower.split('#').next().unwrap_or(&lower);
    let without_query = without_fragment
        .split('?')
        .next()
        .unwrap_or(without_fragment);
    without_query.ends_with(".pdf")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_empty_input_to_home_page() {
        assert_eq!(normalize_url("   "), HOME_PAGE_URL);
    }

    #[test]
    fn normalizes_missing_scheme_to_https() {
        assert_eq!(
            normalize_url("example.com/docs"),
            "https://example.com/docs"
        );
    }

    #[test]
    fn preserves_existing_scheme() {
        assert_eq!(
            normalize_url("http://localhost:3000"),
            "http://localhost:3000"
        );
    }

    #[test]
    fn detects_pdf_urls_with_query_and_fragment() {
        assert!(is_pdf_url(
            "https://example.com/report.PDF?download=true#page=3"
        ));
    }

    #[test]
    fn ignores_non_pdf_urls() {
        assert!(!is_pdf_url("https://example.com/report.pdf.html"));
    }
}
