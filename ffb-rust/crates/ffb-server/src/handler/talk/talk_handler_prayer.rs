/// 1:1 translation of com.fumbbl.ffb.server.handler.talk.TalkHandlerPrayer.
/// Handles /prayer command — applies a prayer inducement effect.
///
/// Ported using the real prayer subsystem that exists today:
/// `ffb_model::inducement::{bb2020,bb2025}::prayers::Prayers` (roll → `Prayer` lookup,
/// replacing Java's stub `PrayerFactory.forRoll`) and
/// `ffb_engine::factory::mixed::prayer_handler_factory::PrayerHandlerFactory` +
/// `PrayerHandler` trait (replacing Java's `PrayerHandlerFactory`/`PrayerHandler`).
///
/// Two deliberate deviations from the Java body, both forced by real infra gaps:
///
/// 1. Java resolves `genericParameter = game.getDialogParameter()` and branches on its
///    concrete dialog type (`DialogSelectSkillParameter` for skill-choice prayers,
///    `DialogPlayerChoiceParameter` for player-choice prayers) to read back the coach's
///    dialog answer. `Game` has no `dialog_parameter` field in Rust — the concrete
///    handlers already ported in `ffb-engine` (e.g. `KnuckleDustersHandler`,
///    `IronManHandler`) resolve their target player *inside* `init_effect` via a
///    `PlayerSelector` (random selection) instead of waiting for a coach dialog, so
///    `init_effect` alone fully applies these prayers with no dialog round-trip needed.
///    To stay faithful to the Java *shape* of the method, this port still takes the
///    dialog parameters as explicit optional inputs (mirroring `TalkHandler`'s existing
///    pattern of passing otherwise-missing `GameState` data in as parameters — see
///    `play_sound_after_cooldown`) so the branch exists and is exercised once a dialog
///    system lands; today the caller has nothing to pass, so both are `None`.
/// 2. `PrayerHandler::apply_selection` (the Rust trait) only takes `team_id` — it has no
///    parameter for the resolved `player_id`/`skill`, unlike Java's
///    `handler.applySelection(null, gameState, new PrayerDialogSelection(playerId, skill))`.
///    That is a genuine trait-signature gap (owned by the prayer-handler translation
///    pass), so the resolved `player_id`/`skill` are computed (for the info messages,
///    matching Java's user-visible text exactly) but cannot be threaded into
///    `apply_selection` — that specific call is left narrower than Java's, calling the
///    trait method with only what it accepts.
///
/// Java's trailing `UtilServerGame.syncGameModel(...)` has no wired Rust target yet
/// (see `talk_handler_activated.rs`) — the caller is responsible for it once available.
use std::collections::HashSet;
use ffb_engine::factory::mixed::prayer_handler_factory::PrayerHandlerFactory;
use ffb_model::dialog::dialog_player_choice_parameter::DialogPlayerChoiceParameter;
use ffb_model::dialog::dialog_select_skill_parameter::DialogSelectSkillParameter;
use ffb_model::enums::Rules;
use ffb_model::model::game::Game;
use ffb_model::model::team::Team;
use ffb_model::util::rng::GameRng;
use crate::handler::talk::talk_handler::TalkHandler;
use crate::handler::talk::talk_requirements::{Client, Environment};

pub struct TalkHandlerPrayer {
    base: TalkHandler,
}

impl TalkHandlerPrayer {
    /// Java: `TalkHandlerPrayer()` — `super("/prayer", 0, Client.PLAYER, Environment.TEST_GAME)`.
    pub fn new() -> Self {
        let mut commands = HashSet::new();
        commands.insert("/prayer".to_string());
        Self {
            base: TalkHandler::new(commands, 0, Client::Player, Environment::TestGame, HashSet::new()),
        }
    }

    pub fn base(&self) -> &TalkHandler { &self.base }

    /// Java: `game.<PrayerFactory>getFactory(FactoryType.Factory.PRAYER).forRoll(roll)`,
    /// then `prayer.getName()`. Returns `(enumConstantName, displayName)` so the caller
    /// gets both the `PrayerHandlerFactory::for_prayer` lookup key and the user-visible text.
    fn prayer_for_roll(rules: Rules, roll: i32) -> Option<(String, String)> {
        match rules {
            Rules::Bb2025 => ffb_model::inducement::bb2025::prayers::Prayers::new()
                .get_prayer(roll)
                .map(|p| (p.name().to_string(), p.get_name().to_string())),
            Rules::Bb2020 => ffb_model::inducement::bb2020::prayers::Prayers::new()
                .get_prayer(roll)
                .map(|p| (p.name().to_string(), p.get_name().to_string())),
            _ => None,
        }
    }

    /// Java: `handle(FantasyFootballServer, GameState, String[], Team, Session)`.
    /// `dialog_select_skill`/`dialog_player_choice` stand in for
    /// `game.getDialogParameter()` — see the module doc comment above. Returns the info
    /// message(s) Java would have sent via `communication.sendPlayerTalk` (Java can send
    /// more than one message in the success path: the "Adding ..." line plus the trailing
    /// "Added prayer ..." line).
    pub fn handle(
        &self,
        game: &mut Game,
        commands: &[String],
        team: &Team,
        rng: &mut GameRng,
        dialog_select_skill: Option<&DialogSelectSkillParameter>,
        dialog_player_choice: Option<&DialogPlayerChoiceParameter>,
    ) -> Vec<String> {
        if commands.len() < 2 {
            return vec!["Prayer roll/Number missing.".to_string()];
        }

        let roll: i32 = match commands[1].parse() {
            Ok(r) => r,
            Err(_) => return vec![format!("{} is not a number.", commands[1])],
        };

        let (prayer_key, prayer_display_name) = match Self::prayer_for_roll(game.rules, roll) {
            Some(p) => p,
            None => {
                return vec![format!(
                    "No prayer found for {} must be between 1 and 16 (inclusive).",
                    commands[1]
                )];
            }
        };

        let mut handler_factory = PrayerHandlerFactory::new();
        handler_factory.initialize(game.rules);
        let handler = match handler_factory.for_prayer(&prayer_key) {
            Some(h) => h,
            None => return vec![format!("No handler found for prayer {prayer_display_name}")],
        };

        let mut prayer_state = std::mem::take(&mut game.prayer_state);
        handler.init_effect(&mut prayer_state, game, rng, &team.id);
        game.prayer_state = prayer_state;

        let requires_argument = matches!(roll, 4 | 5 | 8 | 16);
        if commands.len() < 3 && requires_argument {
            return vec![format!("Prayer {prayer_display_name} requires an additional parameter.")];
        }

        let mut info: Vec<String> = Vec::new();
        let mut player_id: Option<String> = None;

        if let Some(dialog_parameter) = dialog_select_skill {
            let player_id_from_dialog = dialog_parameter.get_player_id();
            let player = player_id_from_dialog
                .filter(|id| !id.is_empty())
                .and_then(|id| game.player(id));
            let player = match player {
                Some(p) => p,
                None => return vec!["Could not select a random player.".to_string()],
            };
            if dialog_parameter.get_skills().is_empty() {
                return vec![format!(
                    "Randomly selected player {} already has all primary skills.",
                    player.name
                )];
            } else {
                let player_name = player.name.clone();
                let requested_skill_name = commands[2].replace('_', " ");
                // Java compares against `Skill.getName()` (a proper display name, e.g.
                // "Mighty Blow"). `SkillId` has no display-name table in Rust yet, only
                // `class_name()` (e.g. "MightyBlow"), so this matches against that with
                // spaces stripped from both sides instead.
                let found = dialog_parameter.get_skills().iter().find(|s| {
                    s.class_name().eq_ignore_ascii_case(&requested_skill_name.replace(' ', ""))
                });
                match found {
                    Some(skill) => {
                        player_id = player_id_from_dialog.map(|s| s.to_string());
                        info.push(format!("Adding {} to player {player_name}.", skill.class_name()));
                    }
                    None => {
                        return vec![format!(
                            "Skill {requested_skill_name} is not available for this player."
                        )];
                    }
                }
            }
        } else if let Some(dialog_parameter) = dialog_player_choice {
            let player_number: i32 = match commands[2].parse() {
                Ok(n) => n,
                Err(_) => return vec![format!("{} is not a number.", commands[2])],
            };
            let found_player = dialog_parameter
                .get_player_ids()
                .iter()
                .filter_map(|id| game.player(id))
                .find(|p| p.nr == player_number);
            match found_player {
                Some(p) => {
                    player_id = Some(p.id.clone());
                    info.push(format!("Adding effect of {prayer_display_name} to player {}.", p.name));
                }
                None => {
                    return vec![format!(
                        "Player with #{} is not eligible for selection.",
                        commands[2]
                    )];
                }
            }
        }

        if player_id.filter(|id| !id.is_empty()).is_some() {
            // Java: `handler.applySelection(null, gameState, new PrayerDialogSelection(playerId, skill))`.
            // The Rust `PrayerHandler::apply_selection` trait method only accepts `team_id`
            // (no player_id/skill parameter yet) — see the module doc comment above.
            let mut prayer_state = std::mem::take(&mut game.prayer_state);
            handler.apply_selection(&mut prayer_state, game, &team.id);
            game.prayer_state = prayer_state;
        } else if requires_argument {
            return vec!["No eligible players/skills".to_string()];
        }

        info.push(format!("Added prayer {prayer_display_name} for coach {}.", team.coach));
        info
    }
}

impl Default for TalkHandlerPrayer {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerGender, PlayerType, SkillId};
    use ffb_model::model::player::Player;
    use std::collections::HashSet as Set;

    fn team(id: &str) -> Team {
        Team {
            id: id.into(), name: "Team".into(), race: "Human".into(), roster_id: "human".into(),
            coach: "Coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
            assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players: vec![], vampire_lord: false, necromancer: false,
        }
    }

    fn make_player(id: &str, nr: i32) -> Player {
        Player {
            id: id.into(), name: format!("Player{nr}"), nr, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Set::new(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None, is_big_guy: false,
            ..Default::default()
        }
    }

    fn game() -> Game {
        Game::new(team("home"), team("away"), Rules::Bb2025)
    }

    #[test]
    fn construct() { let _ = TalkHandlerPrayer::new(); }

    #[test]
    fn handle_returns_missing_roll_message() {
        let h = TalkHandlerPrayer::new();
        let mut g = game();
        let commands = vec!["/prayer".to_string()];
        let info = h.handle(&mut g, &commands, &team("home"), &mut GameRng::new(0), None, None);
        assert_eq!(info, vec!["Prayer roll/Number missing.".to_string()]);
    }

    #[test]
    fn handle_returns_not_a_number_message() {
        let h = TalkHandlerPrayer::new();
        let mut g = game();
        let commands = vec!["/prayer".to_string(), "abc".to_string()];
        let info = h.handle(&mut g, &commands, &team("home"), &mut GameRng::new(0), None, None);
        assert_eq!(info, vec!["abc is not a number.".to_string()]);
    }

    #[test]
    fn handle_returns_no_prayer_found_for_out_of_range_roll() {
        let h = TalkHandlerPrayer::new();
        let mut g = game();
        let commands = vec!["/prayer".to_string(), "99".to_string()];
        let info = h.handle(&mut g, &commands, &team("home"), &mut GameRng::new(0), None, None);
        assert_eq!(info.len(), 1);
        assert!(info[0].contains("No prayer found for 99"));
    }

    #[test]
    fn handle_applies_non_argument_prayer_immediately() {
        let h = TalkHandlerPrayer::new();
        let mut g = game();
        // roll 13 = FOULING_FRENZY (bb2025), no dialog argument required.
        let commands = vec!["/prayer".to_string(), "13".to_string()];
        let info = h.handle(&mut g, &commands, &team("home"), &mut GameRng::new(0), None, None);
        assert_eq!(info.len(), 1);
        assert!(info[0].contains("Added prayer Fouling Frenzy for coach Coach."));
    }

    #[test]
    fn handle_requires_argument_prayer_without_third_command_returns_message() {
        let h = TalkHandlerPrayer::new();
        let mut g = game();
        // roll 4 = IRON_MAN (bb2025), requires an argument per Java's [4,5,8,16] list.
        let commands = vec!["/prayer".to_string(), "4".to_string()];
        let info = h.handle(&mut g, &commands, &team("home"), &mut GameRng::new(0), None, None);
        assert_eq!(info.len(), 1);
        assert!(info[0].contains("requires an additional parameter"));
    }

    #[test]
    fn handle_requires_argument_prayer_with_no_dialog_selection_reports_no_eligible() {
        let h = TalkHandlerPrayer::new();
        let mut g = game();
        // roll 4 = IRON_MAN; third arg present but no dialog parameter wired → no eligible players/skills.
        let commands = vec!["/prayer".to_string(), "4".to_string(), "1".to_string()];
        let info = h.handle(&mut g, &commands, &team("home"), &mut GameRng::new(0), None, None);
        assert_eq!(info, vec!["No eligible players/skills".to_string()]);
    }

    #[test]
    fn handle_select_skill_dialog_reports_adding_skill_and_final_message() {
        let h = TalkHandlerPrayer::new();
        let mut g = game();
        g.team_home.players.push(make_player("p1", 1));
        let dialog = DialogSelectSkillParameter {
            player_id: Some("p1".to_string()),
            skills: vec![SkillId::MightyBlow],
            skill_choice_mode: None,
        };
        // roll 5 = KNUCKLE_DUSTERS (bb2025), requires argument.
        let commands = vec!["/prayer".to_string(), "5".to_string(), "Mighty_Blow".to_string()];
        let info = h.handle(&mut g, &commands, &team("home"), &mut GameRng::new(0), Some(&dialog), None);
        assert_eq!(info.len(), 2);
        assert!(info[0].contains("Adding MightyBlow to player Player1."));
        assert!(info[1].contains("Added prayer Knuckle Dusters for coach Coach."));
    }

    #[test]
    fn handle_player_choice_dialog_reports_adding_effect_and_final_message() {
        let h = TalkHandlerPrayer::new();
        let mut g = game();
        g.team_home.players.push(make_player("p1", 7));
        let dialog = DialogPlayerChoiceParameter {
            team_id: Some("home".to_string()),
            player_choice_mode: None,
            player_ids: vec!["p1".to_string()],
            descriptions: vec![],
            max_selects: 1,
            min_selects: 1,
        };
        // roll 5 = KNUCKLE_DUSTERS (bb2025), requires argument; commands[2] = player number 7.
        let commands = vec!["/prayer".to_string(), "5".to_string(), "7".to_string()];
        let info = h.handle(&mut g, &commands, &team("home"), &mut GameRng::new(0), None, Some(&dialog));
        assert_eq!(info.len(), 2);
        assert!(info[0].contains("Adding effect of Knuckle Dusters to player Player7."));
        assert!(info[1].contains("Added prayer Knuckle Dusters for coach Coach."));
    }
}
