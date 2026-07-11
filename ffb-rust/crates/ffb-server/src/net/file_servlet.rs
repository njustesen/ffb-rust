/// 1:1 translation of com.fumbbl.ffb.server.net.FileServlet.
///
/// Java: extends Jetty's `DefaultServlet` (static file serving off
/// `resourceBase`/`httpDir`), overriding `doGet` only to add a TRACE log line
/// after delegating to the default file-serving behavior. Mounted at `/*`
/// (`FantasyFootballServer.start()`). Rust has no `DefaultServlet`; this reads
/// the requested file from `root_dir` directly and returns 404 when absent —
/// the file-serving behavior Jetty's default handler provides for a plain
/// static asset request (directory listings / welcome-file resolution are out
/// of scope; nothing in this server serves a directory).
use std::path::{Path, PathBuf};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

pub struct FileServlet;

impl FileServlet {
    pub fn new() -> Self {
        Self
    }

    /// Java:
    /// ```java
    /// protected void doGet(HttpServletRequest pRequest, HttpServletResponse pResponse)
    ///         throws ServletException, IOException {
    ///     super.doGet(pRequest, pResponse);
    ///     if (fServer.getDebugLog().isLogging(IServerLogLevel.TRACE)) {
    ///         fServer.getDebugLog().logWithOutGameId(IServerLogLevel.TRACE, "get " + pRequest.getRequestURL());
    ///     }
    /// }
    /// ```
    /// `root_dir` is the Rust stand-in for the servlet's `resourceBase` init
    /// parameter (`httpDir`, set from `IServerProperty.SERVER_BASE_DIR`);
    /// `request_url` is the requested path. Reads and returns the file's bytes
    /// with a 200, or 404 if it doesn't exist — the observable behavior of
    /// `super.doGet()` for a plain file request. The TRACE log always fires
    /// here (this crate's `log` crate doesn't expose a level-is-enabled check
    /// per request the way `DebugLog.isLogging` does, so the `log::trace!`
    /// macro's own level filtering plays that role instead).
    pub async fn do_get(&self, root_dir: &Path, request_url: &str) -> Response {
        let path = Self::resolve_path(root_dir, request_url);
        let response = match tokio::fs::read(&path).await {
            Ok(bytes) => (StatusCode::OK, bytes).into_response(),
            Err(_) => StatusCode::NOT_FOUND.into_response(),
        };
        log::trace!("get {}", request_url);
        response
    }

    /// Java: implicit in `DefaultServlet` — resolves the request path against
    /// `resourceBase`. `request_url`'s leading `/` is stripped so `Path::join`
    /// treats it as relative to `root_dir` rather than replacing it outright.
    fn resolve_path(root_dir: &Path, request_url: &str) -> PathBuf {
        root_dir.join(request_url.trim_start_matches('/'))
    }
}

impl Default for FileServlet {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() {
        let _ = FileServlet::new();
    }

    #[test]
    fn default() {
        let _ = FileServlet::default();
    }

    #[test]
    fn resolve_path_strips_leading_slash() {
        let root = Path::new("/var/www");
        let resolved = FileServlet::resolve_path(root, "/index.html");
        assert_eq!(resolved, Path::new("/var/www/index.html"));
    }

    #[test]
    fn resolve_path_nested() {
        let root = Path::new("/var/www");
        let resolved = FileServlet::resolve_path(root, "/assets/app.js");
        assert_eq!(resolved, Path::new("/var/www/assets/app.js"));
    }

    #[tokio::test]
    async fn do_get_existing_file_returns_200_with_body() {
        let dir = std::env::temp_dir().join(format!("ffb_file_servlet_test_{}", std::process::id()));
        tokio::fs::create_dir_all(&dir).await.unwrap();
        let file_path = dir.join("hello.txt");
        tokio::fs::write(&file_path, b"hello world").await.unwrap();

        let servlet = FileServlet::new();
        let response = servlet.do_get(&dir, "/hello.txt").await;
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(&body[..], b"hello world");

        tokio::fs::remove_dir_all(&dir).await.unwrap();
    }

    #[tokio::test]
    async fn do_get_missing_file_returns_404() {
        let dir = std::env::temp_dir().join(format!("ffb_file_servlet_test_missing_{}", std::process::id()));
        let servlet = FileServlet::new();
        let response = servlet.do_get(&dir, "/does-not-exist.txt").await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
