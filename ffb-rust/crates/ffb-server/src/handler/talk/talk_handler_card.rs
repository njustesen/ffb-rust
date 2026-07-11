/// 1:1 translation of com.fumbbl.ffb.server.handler.talk.TalkHandlerCard.
/// Handles /card command — adds or removes inducement cards for a team.
///
/// Java resolves `commands[2]` via
/// `CardFactory.forShortName(commands[2].replace('_', ' '))` to a `Card` object,
/// then uses `card.getName()`. The ported `CardFactory` (ffb-model) is a bare
/// stub with no lookup methods yet (Phase 4 dependency noted in its own file) —
/// there is no Rust equivalent to call. This handler therefore takes the
/// already-resolved card display name as an explicit parameter (the caller
/// performs the short-name lookup once `CardFactory` is wired), the same
/// adaptation pattern `TalkHandler::play_sound_after_cooldown` uses for the
/// missing per-coach cooldown map. `card_name = None` mirrors Java's
/// `card == null` early return.
///
/// Java's `handle()` also calls `UtilServerGame.syncGameModel(...)` at the end
/// — the caller's responsibility once sync-to-client infra exists (see
/// `talk_handler_activated.rs` for the same documented adaptation).
use ffb_model::model::game::Game;
use ffb_model::model::team::Team;

const ADD: &str = "add";
const REMOVE: &str = "remove";

pub struct TalkHandlerCard;

impl TalkHandlerCard {
    pub fn new() -> Self { Self }

    /// Java: `handle(FantasyFootballServer, GameState, String[], Team, Session)` —
    /// adds/removes a card from the coach's turn data; returns the info
    /// message(s) Java would have sent via `communication.sendPlayerTalk`.
    pub fn handle(&self, game: &mut Game, commands: &[String], team: &Team, card_name: Option<&str>) -> Vec<String> {
        if commands.len() <= 2 {
            return Vec::new();
        }
        let card_name = match card_name {
            Some(name) => name,
            None => return Vec::new(),
        };

        let home_coach = game.is_home_team(&team.id);
        let coach = if home_coach { game.team_home.coach.clone() } else { game.team_away.coach.clone() };
        let turn_data = if home_coach { &mut game.turn_data_home } else { &mut game.turn_data_away };

        let mut info = Vec::new();
        if commands[1] == ADD {
            turn_data.inducement_set.add_available_card(card_name);
            info.push(format!("Added card {card_name} for coach {coach}."));
        }
        if commands[1] == REMOVE {
            turn_data.inducement_set.remove_available_card(card_name);
            info.push(format!("Removed card {card_name} for coach {coach}."));
        }
        info
    }
}

impl Default for TalkHandlerCard {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;

    fn make_team(id: &str, coach: &str) -> Team {
        Team {
            id: id.into(), name: "Team".into(), race: "Human".into(), roster_id: "human".into(),
            coach: coach.into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
            assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players: vec![], vampire_lord: false, necromancer: false,
        }
    }

    #[test]
    fn construct() { let _ = TalkHandlerCard::new(); }

    #[test]
    fn handle_returns_empty_when_not_enough_parts() {
        let h = TalkHandlerCard::new();
        let home = make_team("home", "Alice");
        let mut game = Game::new(home.clone(), make_team("away", "Bob"), Rules::Bb2025);
        let commands = vec!["/card".to_string(), "add".to_string()];
        let info = h.handle(&mut game, &commands, &home, Some("Chop Block"));
        assert!(info.is_empty());
    }

    #[test]
    fn handle_returns_empty_when_card_not_found() {
        let h = TalkHandlerCard::new();
        let home = make_team("home", "Alice");
        let mut game = Game::new(home.clone(), make_team("away", "Bob"), Rules::Bb2025);
        let commands = vec!["/card".to_string(), "add".to_string(), "chop_block".to_string()];
        let info = h.handle(&mut game, &commands, &home, None);
        assert!(info.is_empty());
    }

    #[test]
    fn handle_add_adds_card_for_home_coach() {
        let h = TalkHandlerCard::new();
        let home = make_team("home", "Alice");
        let mut game = Game::new(home.clone(), make_team("away", "Bob"), Rules::Bb2025);
        let commands = vec!["/card".to_string(), "add".to_string(), "chop_block".to_string()];
        let info = h.handle(&mut game, &commands, &home, Some("Chop Block"));
        assert_eq!(info, vec!["Added card Chop Block for coach Alice.".to_string()]);
        assert!(game.turn_data_home.inducement_set.is_available("Chop Block"));
    }

    #[test]
    fn handle_remove_removes_card_for_away_coach() {
        let h = TalkHandlerCard::new();
        let home = make_team("home", "Alice");
        let away = make_team("away", "Bob");
        let mut game = Game::new(home, away.clone(), Rules::Bb2025);
        game.turn_data_away.inducement_set.add_available_card("Chop Block");
        let commands = vec!["/card".to_string(), "remove".to_string(), "chop_block".to_string()];
        let info = h.handle(&mut game, &commands, &away, Some("Chop Block"));
        assert_eq!(info, vec!["Removed card Chop Block for coach Bob.".to_string()]);
        assert!(!game.turn_data_away.inducement_set.is_available("Chop Block"));
    }
}
