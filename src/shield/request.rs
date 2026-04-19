use reqwest::Url;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ShieldRequestContext {
    Navigation,
    NewWindow,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ShieldResourceType {
    Document,
    Pdf,
    Image,
    Script,
    Stylesheet,
    Media,
    Font,
    Data,
    Archive,
    Json,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ShieldRequest {
    tab_id: usize,
    url: String,
    host: Option<String>,
    context: ShieldRequestContext,
    resource_type: ShieldResourceType,
    source_url: Option<String>,
}

impl ShieldRequestContext {
    pub(crate) fn label(&self) -> &'static str {
        match self {
            Self::Navigation => "Navigation",
            Self::NewWindow => "New Window",
        }
    }
}

impl ShieldResourceType {
    pub(crate) fn label(&self) -> &'static str {
        match self {
            Self::Document => "Document",
            Self::Pdf => "PDF",
            Self::Image => "Image",
            Self::Script => "Script",
            Self::Stylesheet => "Stylesheet",
            Self::Media => "Media",
            Self::Font => "Font",
            Self::Data => "Data",
            Self::Archive => "Archive",
            Self::Json => "JSON",
        }
    }
}

impl ShieldRequest {
    pub(crate) fn new(
        tab_id: usize,
        url: impl Into<String>,
        context: ShieldRequestContext,
        source_url: Option<String>,
    ) -> Self {
        let url = url.into();

        Self {
            tab_id,
            host: extract_host(&url),
            resource_type: classify_resource_type(&url, context),
            url,
            context,
            source_url,
        }
    }

    pub(crate) fn tab_id(&self) -> usize {
        self.tab_id
    }

    pub(crate) fn url(&self) -> &str {
        &self.url
    }

    pub(crate) fn host(&self) -> Option<&str> {
        self.host.as_deref()
    }

    pub(crate) fn context(&self) -> ShieldRequestContext {
        self.context
    }

    pub(crate) fn resource_type(&self) -> ShieldResourceType {
        self.resource_type
    }

    pub(crate) fn source_url(&self) -> Option<&str> {
        self.source_url.as_deref()
    }
}

pub(crate) fn is_internal_browser_url(url: &str) -> bool {
    let normalized = url.trim().to_ascii_lowercase();
    normalized.starts_with("grimley://") || normalized.starts_with("about:blank#grimley-")
}

fn extract_host(url: &str) -> Option<String> {
    Url::parse(url)
        .ok()
        .and_then(|parsed| parsed.host_str().map(str::to_ascii_lowercase))
}

fn classify_resource_type(url: &str, context: ShieldRequestContext) -> ShieldResourceType {
    let path = Url::parse(url)
        .ok()
        .map(|parsed| parsed.path().to_ascii_lowercase())
        .unwrap_or_else(|| url.to_ascii_lowercase());

    if has_extension(&path, &["pdf"]) {
        ShieldResourceType::Pdf
    } else if has_extension(
        &path,
        &["png", "jpg", "jpeg", "gif", "webp", "svg", "ico", "bmp", "avif"],
    ) {
        ShieldResourceType::Image
    } else if has_extension(&path, &["js", "mjs", "cjs"]) {
        ShieldResourceType::Script
    } else if has_extension(&path, &["css"]) {
        ShieldResourceType::Stylesheet
    } else if has_extension(
        &path,
        &["mp3", "mp4", "wav", "ogg", "ogv", "webm", "mov", "avi", "m4a", "flac"],
    ) {
        ShieldResourceType::Media
    } else if has_extension(&path, &["woff", "woff2", "ttf", "otf", "eot"]) {
        ShieldResourceType::Font
    } else if has_extension(&path, &["json"]) {
        ShieldResourceType::Json
    } else if has_extension(&path, &["xml", "csv", "txt"]) {
        ShieldResourceType::Data
    } else if has_extension(&path, &["zip", "rar", "7z", "gz", "tar", "bz2"]) {
        ShieldResourceType::Archive
    } else {
        match context {
            ShieldRequestContext::Navigation | ShieldRequestContext::NewWindow => {
                ShieldResourceType::Document
            }
        }
    }
}

fn has_extension(path: &str, extensions: &[&str]) -> bool {
    extensions
        .iter()
        .any(|extension| path.ends_with(&format!(".{extension}")))
}
