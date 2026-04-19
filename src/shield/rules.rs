use super::request::ShieldRequest;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ShieldRuleAction {
    Block,
    Allow,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ShieldRuleMatch {
    label: String,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct ShieldRuleset {
    rules: Vec<ShieldRule>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ShieldRuleMatches {
    pub(crate) blocked: Vec<ShieldRuleMatch>,
    pub(crate) exception: Option<ShieldRuleMatch>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ShieldRule {
    domain: String,
    label: String,
    action: ShieldRuleAction,
}

impl ShieldRuleAction {
    fn label(&self) -> &'static str {
        match self {
            Self::Block => "Block",
            Self::Allow => "Allow",
        }
    }
}

impl ShieldRuleMatch {
    pub(crate) fn label(&self) -> &str {
        &self.label
    }
}

impl ShieldRuleset {
    pub(crate) fn from_text(input: &str) -> Self {
        let rules = input.lines().filter_map(ShieldRule::parse).collect();
        Self { rules }
    }

    pub(crate) fn len(&self) -> usize {
        self.rules.len()
    }

    pub(crate) fn find_matches(&self, request: &ShieldRequest) -> ShieldRuleMatches {
        let Some(host) = request.host() else {
            return ShieldRuleMatches::default();
        };

        let mut blocked = Vec::new();
        let mut exception = None;

        for rule in &self.rules {
            if !rule.matches_host(host) {
                continue;
            }

            let matched = ShieldRuleMatch {
                label: rule.label.clone(),
            };

            match rule.action {
                ShieldRuleAction::Block => blocked.push(matched),
                ShieldRuleAction::Allow => {
                    if exception.is_none() {
                        exception = Some(matched);
                    }
                }
            }
        }

        ShieldRuleMatches { blocked, exception }
    }
}

impl ShieldRule {
    fn parse(line: &str) -> Option<Self> {
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('!') || trimmed.starts_with('#') {
            return None;
        }

        let (action, pattern) = if let Some(exception) = trimmed.strip_prefix("@@") {
            (ShieldRuleAction::Allow, exception)
        } else {
            (ShieldRuleAction::Block, trimmed)
        };

        let domain = normalize_domain_pattern(pattern)?;

        Some(Self {
            label: format!("{} {}", action.label(), domain),
            domain,
            action,
        })
    }

    fn matches_host(&self, host: &str) -> bool {
        host == self.domain || host.ends_with(&format!(".{}", self.domain))
    }
}

fn normalize_domain_pattern(pattern: &str) -> Option<String> {
    let trimmed = pattern.trim();
    let without_easylist_prefix = trimmed.strip_prefix("||").unwrap_or(trimmed);
    let without_scheme = without_easylist_prefix
        .split("://")
        .nth(1)
        .unwrap_or(without_easylist_prefix);
    let without_path = without_scheme.split('/').next().unwrap_or(without_scheme);
    let without_suffix = without_path.strip_suffix('^').unwrap_or(without_path);
    let normalized = without_suffix
        .trim()
        .trim_matches('.')
        .to_ascii_lowercase();

    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}
