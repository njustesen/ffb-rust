/// 1:1 translation of `com.fumbbl.ffb.util.UtilUrl`.
pub struct UtilUrl;

impl UtilUrl {
    /// Java: `createUrl(String baseUrl, String relativeUrl)`.
    ///
    /// Resolves `relative_url` against `base_url` using simple URL joining rules:
    /// - If base is empty → return relative.
    /// - If relative is empty → return base.
    /// - If relative is absolute (starts with scheme like `http://`) → return relative.
    /// - Otherwise append `relative_url` to the last `/`-separated directory of `base_url`.
    pub fn create_url(base_url: Option<&str>, relative_url: Option<&str>) -> Option<String> {
        match (base_url, relative_url) {
            (None, None) | (Some(""), Some("")) | (None, Some("")) | (Some(""), None) => None,
            (None, Some(r)) | (Some(""), Some(r)) => Some(r.to_string()),
            (Some(b), None) | (Some(b), Some("")) => Some(b.to_string()),
            (Some(b), Some(r)) => {
                // Absolute relative URL (has a scheme)
                if r.contains("://") { return Some(r.to_string()); }
                // Root-relative URL (starts with /)
                if r.starts_with('/') {
                    // Grab scheme + host
                    if let Some(pos) = b.find("://") {
                        let after_scheme = &b[pos + 3..];
                        let host_end = after_scheme.find('/').map(|p| pos + 3 + p).unwrap_or(b.len());
                        return Some(format!("{}{}", &b[..host_end], r));
                    }
                    return Some(r.to_string());
                }
                // Relative URL: join onto directory part of base
                let base_dir = if b.ends_with('/') {
                    b.trim_end_matches('/').to_string()
                } else {
                    // strip last path segment
                    match b.rfind('/') {
                        Some(pos) => b[..pos].to_string(),
                        None => b.to_string(),
                    }
                };
                // Handle `..` at start of relative
                let mut parts: Vec<&str> = base_dir.split('/').collect();
                for seg in r.split('/') {
                    if seg == ".." {
                        // pop last non-empty segment
                        while parts.last() == Some(&"") { parts.pop(); }
                        parts.pop();
                    } else if !seg.is_empty() {
                        parts.push(seg);
                    }
                }
                Some(parts.join("/"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_appended_to_base_dir() {
        let result = UtilUrl::create_url(
            Some("http://fumbbl.com/FUMBBL/Images/"),
            Some("PlayerIcons/amlineman1.gif"),
        );
        assert_eq!(result, Some("http://fumbbl.com/FUMBBL/Images/PlayerIcons/amlineman1.gif".into()));
    }

    #[test]
    fn absolute_relative_wins() {
        let result = UtilUrl::create_url(
            Some("http://fumbbl.com/"),
            Some("http://google.de/icon.gif"),
        );
        assert_eq!(result, Some("http://google.de/icon.gif".into()));
    }

    #[test]
    fn empty_base_returns_relative() {
        let result = UtilUrl::create_url(None, Some("icon.gif"));
        assert_eq!(result, Some("icon.gif".into()));
    }
}
