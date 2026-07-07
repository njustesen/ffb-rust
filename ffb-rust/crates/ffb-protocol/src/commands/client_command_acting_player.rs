use ffb_model::enums::PlayerAction;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandActingPlayer`.
/// Sent to declare which player will act next and what action they perform.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandActingPlayer {
    /// Java: `fPlayerId`
    pub player_id: Option<String>,
    /// Java: `fPlayerAction`
    pub player_action: Option<PlayerAction>,
    /// Java: `jumping`
    pub jumping: bool,
}

impl ClientCommandActingPlayer {
    pub fn new(player_id: impl Into<String>, player_action: PlayerAction, jumping: bool) -> Self {
        Self {
            player_id: Some(player_id.into()),
            player_action: Some(player_action),
            jumping,
        }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn get_player_action(&self) -> Option<PlayerAction> { self.player_action }
    pub fn is_jumping(&self) -> bool { self.jumping }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored() {
        let cmd = ClientCommandActingPlayer::new("p1", PlayerAction::Move, false);
        assert_eq!(cmd.get_player_id(), Some("p1"));
        assert_eq!(cmd.get_player_action(), Some(PlayerAction::Move));
        assert!(!cmd.is_jumping());
    }

    #[test]
    fn jumping_flag() {
        let cmd = ClientCommandActingPlayer::new("p2", PlayerAction::Block, true);
        assert!(cmd.is_jumping());
    }

    #[test]
    fn default_is_empty() {
        let cmd = ClientCommandActingPlayer::default();
        assert!(cmd.player_id.is_none());
        assert!(cmd.player_action.is_none());
        assert!(!cmd.jumping);
    }

    #[test]
    fn debug_format_nonempty() {
        let cmd = ClientCommandActingPlayer::default();
        assert!(!format!("{cmd:?}").is_empty());
    }
}
