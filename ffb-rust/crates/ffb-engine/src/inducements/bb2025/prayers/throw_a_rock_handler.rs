/// 1:1 translation of `com.fumbbl.ffb.server.inducements.bb2025.prayers.ThrowARockHandler`.
/// BB2025: initEffect adds a THROW_ROCK inducement to the praying team's InducementSet.
/// Java lines 22–36: factory.allTypes().stream().filter(hasUsage(THROW_ROCK))... add to inducementSet.
/// BB2025 initEffect adds a THROW_ROCK inducement to the praying team's InducementSet (InducementTypeFactory not ported; skipped in headless).
use ffb_model::model::animation_type::AnimationType;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::inducements::mixed::prayers::prayer_handler::PrayerHandler;
use crate::prayer_state::PrayerState;

pub struct ThrowARockHandler;

impl ThrowARockHandler {
    pub fn new() -> Self { Self }
}

impl Default for ThrowARockHandler {
    fn default() -> Self { Self::new() }
}

impl PrayerHandler for ThrowARockHandler {
    fn handled_prayer_name(&self) -> &'static str { "THROW_A_ROCK" }
    fn animation_type(&self) -> AnimationType { AnimationType::PRAYER_THROW_A_ROCK }
    fn get_name(&self) -> &'static str { "ThrowARockHandler" }
    /// Java: the first InducementTypeFactory type with Usage.THROW_ROCK (`throwARock`): +1 value
    /// if the praying team's turn InducementSet already has it, else a new Inducement(type, 1).
    /// (§Q2.8: this was a no-op, so a Throw a Rock prayer granted nothing.)
    fn init_effect(&self, _prayer_state: &mut PrayerState, game: &mut Game, _rng: &mut GameRng, team_id: &str) -> bool {
        use ffb_model::inducement::usage::Usage;
        let Some(rock) = ffb_model::factory::inducement_type_factory::all_types_in(game.rules)
            .into_iter().find(|t| t.usages.contains(&Usage::THROW_ROCK)) else { return true };
        let set = if game.team_home.id == team_id {
            &mut game.turn_data_home.inducement_set
        } else {
            &mut game.turn_data_away.inducement_set
        };
        match set.get(&rock.id) {
            Some(mut existing) => {
                existing.value += 1;
                set.add_inducement(existing);
            }
            None => set.add_inducement(ffb_model::inducement::inducement::Inducement::new(rock.id.clone(), 1, rock.usages.clone())),
        }
        true
    }
    fn remove_effect_internal(&self, _prayer_state: &mut PrayerState, _game: &mut Game, _team_id: &str) {
        // No persistent state to reverse.
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        use ffb_model::enums::Rules;
        use crate::step::framework::test_team;
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    #[test]
    fn handles_prayer_throw_a_rock() {
        let h = ThrowARockHandler;
        assert!(h.handles_prayer("THROW_A_ROCK"));
        assert!(!h.handles_prayer("OTHER"));
    }

    #[test]
    fn init_effect_returns_true() {
        let h = ThrowARockHandler;
        let mut state = PrayerState::new();
        let mut game = make_game();
        assert!(h.init_effect(&mut state, &mut game, &mut GameRng::new(0), "team1"));
    }

    #[test]
    fn animation_type_is_correct() {
        let h = ThrowARockHandler;
        assert_eq!(h.animation_type(), AnimationType::PRAYER_THROW_A_ROCK);
    }

    #[test]
    fn get_name_returns_handler_name() {
        let h = ThrowARockHandler;
        assert_eq!(h.get_name(), "ThrowARockHandler");
    }

    // NOTE (test equalization): `handles_prayer_is_case_sensitive` pruned - Rust-structural
    // (handles_prayer takes a string; Java's handles(Prayer) is enum-based, a lowercase
    // variant is inexpressible there).
}
