/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandAdminMessage`.
/// Carries one or more admin messages from server to client.
#[derive(Debug, Clone, Default)]
pub struct ServerCommandAdminMessage {
    /// Java: `fMessages` — list of admin message strings.
    pub messages: Vec<String>,
}

impl ServerCommandAdminMessage {
    pub fn new(messages: Vec<String>) -> Self { Self { messages } }
    pub fn get_messages(&self) -> &[String] { &self.messages }
    pub fn add_message(&mut self, message: impl Into<String>) {
        self.messages.push(message.into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_messages() {
        let cmd = ServerCommandAdminMessage::new(vec!["hello".into(), "world".into()]);
        assert_eq!(cmd.get_messages(), &["hello", "world"]);
    }

    #[test]
    fn add_message_appends() {
        let mut cmd = ServerCommandAdminMessage::default();
        cmd.add_message("hi");
        assert_eq!(cmd.messages.len(), 1);
    }
}
