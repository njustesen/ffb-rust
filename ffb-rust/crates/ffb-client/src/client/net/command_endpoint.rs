//! 1:1 translation of `com.fumbbl.ffb.client.net.CommandEndpoint`.
//!
//! DOCUMENTED GAP (per `TRANSLATION_TRACKER.md`'s "Phase ZW.2 Batch B" note): Java's
//! `CommandEndpoint` is a `javax.websocket` `@ClientEndpoint` — its actual networking role
//! (opening a socket, receiving/decompressing/parsing frames, async binary send) is already
//! covered by `crate::connection::ServerConnection`, built on `tokio-tungstenite` — a
//! necessarily different tech stack, not a line-for-line translation candidate. Specifically
//! not ported here, with the reason for each:
//! - `onMessage`/`send`'s real socket I/O, `LZString` (de)compression, and the
//!   `NullPointerException` retry loop while `Rules` isn't initialized: `LZString` has no
//!   Rust port anywhere in this codebase, and the retry loop exists only to work around a
//!   Java-specific initialization race in `fClient.getGame().getRules()` (`FantasyFootballClient`
//!   is the permanently-skipped GUI shell).
//! - `awaitClose`/`CountDownLatch`: pure Java thread-coordination primitive with no
//!   equivalent need in `ServerConnection`'s async/await model.
//! - `FfbSendHandler`: logs a failed async send via `fClient.logError(...)` — a GUI-shell
//!   logging call with nothing to translate.
//!
//! What IS pure, translatable logic (kept 1:1 below):
//! - The constructor's `fCommandCompression` parsing from a `CommonProperty` string value.
//! - `isOpen()`'s session-presence/open check (`(fSession != null) && fSession.isOpen()`),
//!   modeled with a plain `bool` standing in for the untranslatable `javax.websocket.Session`.
//! - `onOpen`/`onClose`'s effect on that state.
//! - `handleNetCommand`'s `SERVER_PONG` branch: the actual ping-time arithmetic
//!   (`received - pongCommand.getTimestamp()`, guarded by `timestamp > 0`), extracted as a
//!   standalone pure function — the `GameTitle`/`invokeLater(GameTitleUpdateTask)` GUI
//!   plumbing around it has no ported target and is left out.

/// Java: `com.fumbbl.ffb.client.net.CommandEndpoint`.
pub struct CommandEndpoint {
    /// Java: `fCommandCompression`.
    command_compression: bool,
    /// Java: `fSession != null` — stands in for the untranslatable `javax.websocket.Session`
    /// (see module doc). `true` once `on_open` has been called and until `on_close`.
    session_open: bool,
}

impl CommandEndpoint {
    /// Java:
    /// ```java
    /// public CommandEndpoint(FantasyFootballClient pClient) {
    ///     fClient = pClient;
    ///     fNetCommandFactory = new NetCommandFactory(pClient.getFactorySource());
    ///     fCloseLatch = new CountDownLatch(1);
    ///     String commandCompressionProperty = fClient.getProperty(CommonProperty.CLIENT_COMMAND_COMPRESSION);
    ///     fCommandCompression = false;
    ///     if (StringTool.isProvided(commandCompressionProperty)) {
    ///         fCommandCompression = Boolean.parseBoolean(commandCompressionProperty);
    ///     }
    /// }
    /// ```
    /// `command_compression_property` stands in for `fClient.getProperty(CommonProperty.CLIENT_COMMAND_COMPRESSION)`.
    pub fn new(command_compression_property: Option<&str>) -> Self {
        let command_compression = match command_compression_property {
            Some(s) if !s.is_empty() => s.parse::<bool>().unwrap_or(false),
            _ => false,
        };
        Self { command_compression, session_open: false }
    }

    /// Java: `@OnOpen public void onOpen(Session session, EndpointConfig unused) { fSession = session; }`.
    pub fn on_open(&mut self) {
        self.session_open = true;
    }

    /// Java: `@OnClose public void onClose(Session ignoredUnused, CloseReason ignored)`.
    /// The `fClient.getUserInterface().socketClosed()` GUI call and `fCloseLatch.countDown()`
    /// (see module doc) are not ported.
    pub fn on_close(&mut self) {
        self.session_open = false;
    }

    /// Java: `public boolean isOpen() { return (fSession != null) && fSession.isOpen(); }`.
    pub fn is_open(&self) -> bool {
        self.session_open
    }

    pub fn is_command_compression(&self) -> bool {
        self.command_compression
    }

    /// Java: `handleNetCommand`'s `SERVER_PONG` branch:
    /// ```java
    /// if (NetCommandId.SERVER_PONG == netCommand.getId()) {
    ///     ServerCommandPong pongCommand = (ServerCommandPong) netCommand;
    ///     if (pongCommand.getTimestamp() > 0) {
    ///         long received = System.currentTimeMillis();
    ///         ... gameTitle.setPingTime(received - pongCommand.getTimestamp()) ...
    ///     }
    /// }
    /// ```
    /// Returns the ping time (`received - timestamp`) that Java would have stored on
    /// `GameTitle`, or `None` when Java's `timestamp > 0` guard fails.
    pub fn pong_ping_time(received_millis: i64, timestamp: i64) -> Option<i64> {
        if timestamp > 0 {
            Some(received_millis - timestamp)
        } else {
            None
        }
    }
}

impl Default for CommandEndpoint {
    fn default() -> Self {
        Self::new(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_defaults_compression_off_when_no_property() {
        let ep = CommandEndpoint::new(None);
        assert!(!ep.is_command_compression());
    }

    #[test]
    fn new_defaults_compression_off_when_empty_property() {
        let ep = CommandEndpoint::new(Some(""));
        assert!(!ep.is_command_compression());
    }

    #[test]
    fn new_parses_true_property() {
        let ep = CommandEndpoint::new(Some("true"));
        assert!(ep.is_command_compression());
    }

    #[test]
    fn new_parses_false_property() {
        let ep = CommandEndpoint::new(Some("false"));
        assert!(!ep.is_command_compression());
    }

    #[test]
    fn is_open_false_before_on_open() {
        let ep = CommandEndpoint::default();
        assert!(!ep.is_open());
    }

    #[test]
    fn on_open_then_on_close_toggles_is_open() {
        let mut ep = CommandEndpoint::default();
        ep.on_open();
        assert!(ep.is_open());
        ep.on_close();
        assert!(!ep.is_open());
    }

    #[test]
    fn pong_ping_time_computes_delta_when_timestamp_positive() {
        assert_eq!(CommandEndpoint::pong_ping_time(1000, 400), Some(600));
    }

    #[test]
    fn pong_ping_time_none_when_timestamp_not_positive() {
        assert_eq!(CommandEndpoint::pong_ping_time(1000, 0), None);
        assert_eq!(CommandEndpoint::pong_ping_time(1000, -5), None);
    }
}
