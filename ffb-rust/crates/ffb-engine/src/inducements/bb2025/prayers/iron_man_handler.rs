/// 1:1 translation of `com.fumbbl.ffb.server.inducements.bb2025.prayers.IronManHandler`.
/// Extends mixed IronManHandler with a BB2025 IronManPlayerSelector (own team RESERVE,
/// filtered to players with armour < 11 — Java's private inner `IronManPlayerSelector`).
/// Selects 1 random eligible player on the praying team, marks prayer, and grants +1 AV.
use ffb_model::enums::{PS_RESERVE, PlayerType, SkillId};
use ffb_model::model::animation_type::AnimationType;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::inducements::bb2025::prayers::player_selector::PlayerSelector;
use crate::inducements::mixed::prayers::iron_man_handler::{self, PRAYER_NAME};
use crate::inducements::mixed::prayers::player_selector::PlayerSelector as PlayerSelectorTrait;
use crate::inducements::mixed::prayers::prayer_handler::PrayerHandler;
use crate::prayer_state::PrayerState;

pub struct IronManHandler;

impl IronManHandler {
    pub fn new() -> Self { Self }
}

impl Default for IronManHandler {
    fn default() -> Self { Self::new() }
}

/// Java: private static class IronManPlayerSelector extends PlayerSelector —
/// same eligibility as the base BB2025 PlayerSelector but additionally filters
/// out players whose armour is already 11 or higher.
struct IronManPlayerSelector;

impl PlayerSelectorTrait for IronManPlayerSelector {
    fn select_players(&self, game: &Game, team_id: &str, nr_of_players: i32, rng: &mut GameRng, added_skills: &[SkillId]) -> Vec<String> {
        let team = if game.team_home.id == team_id {
            &game.team_home
        } else {
            &game.team_away
        };

        let mut eligible: Vec<&str> = team.players.iter()
            .filter(|p| {
                game.field_model.player_state(&p.id)
                    .map_or(false, |s| s.base() == PS_RESERVE)
            })
            .filter(|p| p.player_type != PlayerType::Star)
            .filter(|p| added_skills.is_empty() || !added_skills.iter().all(|s| p.has_skill(*s)))
            .filter(|p| p.armour < 11)
            .map(|p| p.id.as_str())
            .collect();

        // Java: shuffle then remove first for each slot — Fisher-Yates shuffle.
        let n = eligible.len();
        for i in (1..n).rev() {
            let j = rng.range(i + 1);
            eligible.swap(i, j);
        }
        eligible.truncate(nr_of_players as usize);
        eligible.iter().map(|s| s.to_string()).collect()
    }
}

impl PrayerHandler for IronManHandler {
    fn handled_prayer_name(&self) -> &'static str { PRAYER_NAME }
    fn animation_type(&self) -> AnimationType { AnimationType::PRAYER_IRON_MAN }
    fn get_name(&self) -> &'static str { "IronManHandler" }

    /// Java: initEffect — selects 1 RESERVE player (armour < 11) on the praying team, grants +1 AV.
    fn init_effect(&self, prayer_state: &mut PrayerState, game: &mut Game, rng: &mut GameRng, team_id: &str) -> bool {
        iron_man_handler::init_effect(prayer_state, game, rng, team_id, &IronManPlayerSelector)
    }

    fn remove_effect_internal(&self, _prayer_state: &mut PrayerState, game: &mut Game, team_id: &str) {
        iron_man_handler::remove_effect_internal(game, team_id, &PlayerSelector::new());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{Rules, PS_RESERVE, PlayerState};
    use ffb_model::model::player::Player;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use ffb_model::util::rng::GameRng;
    use crate::step::framework::test_team;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    fn add_reserve_player(game: &mut Game, id: &str, armour: i32) {
        game.team_home.players.push(Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            player_status: ffb_model::model::player_status::PlayerStatus::ACTIVE,
            ..Default::default()
        });
        game.field_model.set_player_state(id, PlayerState::new(PS_RESERVE));
    }

    #[test]
    fn handles_prayer_iron_man() {
        let h = IronManHandler;
        assert!(h.handles_prayer("IRON_MAN"));
        assert!(!h.handles_prayer("OTHER"));
    }

    #[test]
    fn init_effect_returns_true() {
        let h = IronManHandler;
        let mut state = PrayerState::new();
        let mut game = make_game();
        assert!(h.init_effect(&mut state, &mut game, &mut GameRng::new(0), "home"));
    }

    #[test]
    fn init_effect_grants_av_bonus_to_reserve_player() {
        let h = IronManHandler;
        let mut state = PrayerState::new();
        let mut game = make_game();
        add_reserve_player(&mut game, "h1", 8);
        h.init_effect(&mut state, &mut game, &mut GameRng::new(0), "home");
        assert_eq!(game.player("h1").unwrap().armour_with_modifiers(), 9);
        assert!(game.field_model.has_prayer_enhancement("h1", PRAYER_NAME));
    }

    #[test]
    fn animation_type_is_correct() {
        let h = IronManHandler;
        assert_eq!(h.animation_type(), AnimationType::PRAYER_IRON_MAN);
    }
    #[test]
    fn does_not_handle_other_prayers() {
        let h = IronManHandler;
        assert!(!h.handles_prayer("PERFECT_PASSING"));
        assert!(!h.handles_prayer(""));
    }

    /// Regression: Java's IronManHandler uses a private IronManPlayerSelector that filters
    /// out players whose armour is already 11 or higher. A player with armour 11 must never
    /// be selected (and thus never receives the enhancement or the +1 AV bonus).
    #[test]
    fn init_effect_excludes_players_with_armour_already_at_11() {
        let h = IronManHandler;
        let mut state = PrayerState::new();
        let mut game = make_game();
        add_reserve_player(&mut game, "maxed", 11);
        h.init_effect(&mut state, &mut game, &mut GameRng::new(0), "home");
        assert!(!game.field_model.has_prayer_enhancement("maxed", PRAYER_NAME));
        assert_eq!(game.player("maxed").unwrap().armour_with_modifiers(), 11);
    }

    /// Regression: with a mix of an armour-11 player and an eligible player, only the
    /// eligible (armour < 11) player should ever be selected.
    #[test]
    fn init_effect_selects_only_eligible_armour_player_from_mixed_pool() {
        let h = IronManHandler;
        let mut state = PrayerState::new();
        let mut game = make_game();
        add_reserve_player(&mut game, "maxed", 11);
        add_reserve_player(&mut game, "eligible", 8);
        h.init_effect(&mut state, &mut game, &mut GameRng::new(0), "home");
        assert!(!game.field_model.has_prayer_enhancement("maxed", PRAYER_NAME));
        assert!(game.field_model.has_prayer_enhancement("eligible", PRAYER_NAME));
        assert_eq!(game.player("eligible").unwrap().armour_with_modifiers(), 9);
    }
}
