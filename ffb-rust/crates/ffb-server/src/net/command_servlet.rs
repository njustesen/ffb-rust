/// 1:1 translation of com.fumbbl.ffb.server.net.CommandServlet.
///
/// Java: a Jetty `WebSocketServlet` + `WebSocketCreator` mounted at `/command/*`
/// (`FantasyFootballServer.start()`) that configures the WebSocket policy and
/// hands out a `CommandSocket` per upgraded connection. Rust has no Jetty
/// servlet container — the WebSocket route is an axum `Router`, and the actual
/// per-connection lifecycle Jetty's `CommandSocket` implemented already exists
/// as `command_socket::ws_handler` / `handle_connection`.
use axum::routing::get;
use axum::Router;
use crate::net::command_socket::{ws_handler, AppState};

/// Java: `factory.getPolicy().setIdleTimeout(10000)` in `configure()`.
/// axum has no per-route idle-timeout knob analogous to Jetty's
/// `WebSocketPolicy`; kept here for documentation/parity with the Java
/// constant since nothing in this crate currently enforces it.
pub const IDLE_TIMEOUT_MILLIS: u64 = 10_000;

/// Java: `/command/*` — the servlet mapping applied in `FantasyFootballServer.start()`.
pub const COMMAND_PATH: &str = "/command";

pub struct CommandServlet {
    pub command_compression: bool,
}

impl CommandServlet {
    /// Java: `CommandServlet(FantasyFootballServer pServer)`. `pServer` itself
    /// isn't stored (only used later, per-request, in `createWebSocket`); the
    /// Rust translation instead takes the already-resolved
    /// `SERVER_COMMAND_COMPRESSION` value, matching `create_web_socket`'s role.
    pub fn new(command_compression: bool) -> Self {
        Self { command_compression }
    }

    /// Java: `configure(WebSocketServletFactory factory)`.
    /// ```java
    /// public void configure(WebSocketServletFactory factory) {
    ///     factory.getPolicy().setIdleTimeout(10000);
    ///     factory.setCreator(this);
    /// }
    /// ```
    /// `setCreator(this)` registers this servlet as the handler for upgrade
    /// requests; the axum equivalent is mounting the WebSocket upgrade route
    /// on the router. Returns the router with `/command` wired to
    /// `command_socket::ws_handler` (the Rust `CommandSocket` equivalent).
    pub fn configure(&self, router: Router<AppState>) -> Router<AppState> {
        router.route(COMMAND_PATH, get(ws_handler))
    }

    /// Java: `createWebSocket(ServletUpgradeRequest, ServletUpgradeResponse)`.
    /// ```java
    /// public Object createWebSocket(ServletUpgradeRequest pRequest, ServletUpgradeResponse pResponse) {
    ///     String commandCompressionProperty = null;
    ///     if (fServer != null) {
    ///         commandCompressionProperty = fServer.getProperty(IServerProperty.SERVER_COMMAND_COMPRESSION);
    ///     }
    ///     boolean commandCompression = false;
    ///     if (StringTool.isProvided(commandCompressionProperty)) {
    ///         commandCompression = Boolean.parseBoolean(commandCompressionProperty);
    ///     }
    ///     return new CommandSocket(fServer, commandCompression);
    /// }
    /// ```
    /// The property lookup + default-`false` fallback already happened at
    /// construction (`CommandServlet::new`), so this simply returns the
    /// resolved flag `command_socket::handle_connection` would need — there is
    /// no per-request "created websocket" object in axum's model; the actual
    /// connection object is the `axum::extract::ws::WebSocket` handed to
    /// `ws_handler` by the framework itself.
    pub fn create_web_socket(&self) -> bool {
        self.command_compression
    }
}

impl Default for CommandServlet {
    fn default() -> Self {
        Self::new(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() {
        let _ = CommandServlet::new(false);
    }

    #[test]
    fn construct_with_compression() {
        let s = CommandServlet::new(true);
        assert!(s.command_compression);
    }

    #[test]
    fn configure_mounts_command_route() {
        let servlet = CommandServlet::new(false);
        let router: Router<AppState> = Router::new();
        let _configured: Router<AppState> = servlet.configure(router);
    }

    #[test]
    fn create_web_socket_returns_compression_flag() {
        assert!(!CommandServlet::new(false).create_web_socket());
        assert!(CommandServlet::new(true).create_web_socket());
    }

    #[test]
    fn idle_timeout_matches_java_constant() {
        assert_eq!(IDLE_TIMEOUT_MILLIS, 10_000);
    }

    #[test]
    fn command_path_matches_java_servlet_mapping() {
        assert_eq!(COMMAND_PATH, "/command");
    }

    #[test]
    fn default() {
        let s = CommandServlet::default();
        assert!(!s.command_compression);
    }
}
