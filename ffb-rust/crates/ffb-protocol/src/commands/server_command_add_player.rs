use ffb_model::model::player_state::PlayerState;
use ffb_model::model::roster_player::RosterPlayer;
use ffb_model::model::send_to_box_reason::SendToBoxReason;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandAddPlayer`.
/// Adds a player to the client's view of the game.
#[derive(Debug, Clone, Default)]
pub struct ServerCommandAddPlayer {
    /// Java: `fTeamId` — team this player belongs to.
    pub team_id: String,
    /// Java: `fPlayer` — the roster player being added.
    pub player: RosterPlayer,
    /// Java: `fPlayerState` — initial player state.
    pub player_state: PlayerState,
    /// Java: `fSendToBoxReason` — reason for box placement (if any).
    pub send_to_box_reason: Option<SendToBoxReason>,
    /// Java: `fSendToBoxTurn` — turn number when sent to box.
    pub send_to_box_turn: i32,
}

impl ServerCommandAddPlayer {
    pub fn new(
        team_id: impl Into<String>,
        player: RosterPlayer,
        player_state: PlayerState,
        send_to_box_reason: Option<SendToBoxReason>,
        send_to_box_turn: i32,
    ) -> Self {
        Self {
            team_id: team_id.into(),
            player,
            player_state,
            send_to_box_reason,
            send_to_box_turn,
        }
    }
    pub fn get_team_id(&self) -> &str { &self.team_id }
    pub fn get_player(&self) -> &RosterPlayer { &self.player }
    pub fn get_player_state(&self) -> &PlayerState { &self.player_state }
    pub fn get_send_to_box_reason(&self) -> Option<SendToBoxReason> { self.send_to_box_reason }
    pub fn get_send_to_box_turn(&self) -> i32 { self.send_to_box_turn }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored() {
        let cmd = ServerCommandAddPlayer::new(
            "team1",
            RosterPlayer::default(),
            PlayerState::default(),
            Some(SendToBoxReason::FOUL_BAN),
            3,
        );
        assert_eq!(cmd.get_team_id(), "team1");
        assert_eq!(cmd.get_send_to_box_reason(), Some(SendToBoxReason::FOUL_BAN));
        assert_eq!(cmd.get_send_to_box_turn(), 3);
    }

    #[test]
    fn default_no_box() {
        let cmd = ServerCommandAddPlayer::default();
        assert!(cmd.team_id.is_empty());
        assert!(cmd.send_to_box_reason.is_none());
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ServerCommandAddPlayer::default()).is_empty());
    }

}
