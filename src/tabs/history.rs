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

#[cfg(test)]
mod tests {
    use super::BrowserHistory;

    #[test]
    fn tracks_back_and_forward_navigation() {
        let mut history = BrowserHistory::new("https://grimley.dev");

        history.navigate_to("https://example.com");
        history.navigate_to("https://docs.rs");

        assert_eq!(history.go_back().as_deref(), Some("https://example.com"));
        assert_eq!(history.go_back().as_deref(), Some("https://grimley.dev"));
        assert_eq!(history.go_forward().as_deref(), Some("https://example.com"));
        assert_eq!(history.go_forward().as_deref(), Some("https://docs.rs"));
    }

    #[test]
    fn clears_forward_stack_after_new_navigation() {
        let mut history = BrowserHistory::new("https://grimley.dev");

        history.navigate_to("https://example.com");
        history.navigate_to("https://docs.rs");
        history.go_back();
        history.navigate_to("https://rust-lang.org");

        assert!(history.can_go_back());
        assert!(!history.can_go_forward());
    }

    #[test]
    fn does_not_duplicate_current_url_when_navigating_to_same_page() {
        let mut history = BrowserHistory::new("https://grimley.dev");

        history.navigate_to("https://grimley.dev");

        assert!(!history.can_go_back());
        assert!(!history.can_go_forward());
    }
}
