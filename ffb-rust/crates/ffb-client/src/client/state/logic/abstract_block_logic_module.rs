//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.AbstractBlockLogicModule`.
//!
//! Java's `AbstractBlockLogicModule` is an abstract `LogicModule` subclass providing a
//! concrete `getId()` (`ClientStateId.BLOCK`) and `endTurn()` override, plus one small
//! helper (`isSufferingBloodLust`). Translated as a set of free functions/default-method
//! helpers that a concrete `LogicModule` implementer can call from its own `get_id`/`end_turn`
//! bodies (Rust traits can't share a default across an unrelated blanket impl the way Java's
//! abstract class shares state with `this`, so each concrete block-logic module repeats the
//! one-line delegation — same convention already used for `BaseLogicPlugin`).

use ffb_model::enums::ClientStateId;
use ffb_model::model::acting_player::ActingPlayer;

use crate::client::fantasy_football_client::FantasyFootballClient;
use crate::client::state::logic::client_action::ClientAction;
use crate::client::state::logic::logic_module::LogicModule;

/// java: `AbstractBlockLogicModule.getId()` — always `ClientStateId.BLOCK`.
pub fn get_id() -> ClientStateId {
    ClientStateId::Block
}

/// java: `AbstractBlockLogicModule.endTurn()`
pub fn end_turn<M: LogicModule + ?Sized>(module: &mut M, client: &mut FantasyFootballClient) {
    let (turn_mode, player_id) = match client.game() {
        Some(game) => (game.turn_mode, game.acting_player.player_id.clone()),
        None => return,
    };
    if let Some(player_id) = player_id {
        if let Some(player) = client.game().and_then(|g| g.player(&player_id)).cloned() {
            module.perform(client, &player, ClientAction::END_MOVE);
        }
    }
    client.communication_mut().send_end_turn(turn_mode);
}

/// java: `AbstractBlockLogicModule.isSufferingBloodLust(ActingPlayer actingPlayer)`
pub fn is_suffering_blood_lust(acting_player: &ActingPlayer) -> bool {
    acting_player.suffering_blood_lust
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_id_is_always_block() {
        assert_eq!(get_id(), ClientStateId::Block);
    }

    #[test]
    fn is_suffering_blood_lust_reads_flag() {
        let mut ap = ActingPlayer::new();
        assert!(!is_suffering_blood_lust(&ap));
        ap.suffering_blood_lust = true;
        assert!(is_suffering_blood_lust(&ap));
    }
}
