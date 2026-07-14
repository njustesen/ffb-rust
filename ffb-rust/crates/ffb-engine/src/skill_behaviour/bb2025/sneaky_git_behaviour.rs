/// 1:1 translation of com.fumbbl.ffb.server.skillbehaviour.bb2025.SneakyGitBehaviour.
///
/// SneakyGit registers two step modifiers:
///   1. StepEjectPlayer - switches ejection from BANNED to KNOCKED_OUT when the
///      SNEAKY_GIT_BAN_TO_KO game option is enabled; skips ejection entirely when
///      ArgueTheCall succeeded and the player was not already a casualty.
///   2. StepReferee - skips the doubles check on the armour roll when SneakyGit is
///      active and the armour was not broken.
///
/// Both modifiers are headless-safe: game-option/prayer-state branches are marked
/// with // headless: and fall back to safe defaults.
use crate::model::skill_behaviour::SkillBehaviour as SbContainer;
use crate::model::step_modifier::StepModifierTrait;
use crate::step::framework::StepId;
use crate::skill_behaviour::registry::SkillRegistry;
use ffb_model::enums::{SkillId, SendToBoxReason, PS_KNOCKED_OUT, PS_BANNED};
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;

// -- HookState for StepEjectPlayer ---------------------------------------------

/// Step-local state exposed to step modifiers by StepEjectPlayer.
/// Java: StepEjectPlayer.StepState
pub struct StepEjectPlayerHookState {
    pub argue_the_call_successful: Option<bool>,
    pub officious_ref: bool,
}

impl StepEjectPlayerHookState {
    pub fn new() -> Self {
        Self {
            argue_the_call_successful: None,
            officious_ref: false,
        }
    }
}

impl Default for StepEjectPlayerHookState {
    fn default() -> Self { Self::new() }
}

// -- HookState for StepReferee -------------------------------------------------

/// Step-local state exposed to step modifiers by StepReferee.
/// Java: StepReferee.StepState
pub struct StepRefereeHookState {
    pub goto_label_on_end: String,
    pub referee_spots_foul: Option<bool>,
}

impl StepRefereeHookState {
    pub fn new() -> Self {
        Self {
            goto_label_on_end: String::new(),
            referee_spots_foul: None,
        }
    }
}

impl Default for StepRefereeHookState {
    fn default() -> Self { Self::new() }
}

// -- SneakyGitBehaviour --------------------------------------------------------

pub struct SneakyGitBehaviour;

impl SneakyGitBehaviour {
    pub fn new() -> Self { Self }

    pub fn register_into(registry: &mut SkillRegistry) {
        let mut sb = SbContainer::new();
        sb.register_step_modifier(Box::new(SneakyGitEjectPlayerModifier));
        sb.register_step_modifier(Box::new(SneakyGitRefereeModifier));
        registry.register(SkillId::SneakyGit, sb);
    }
}

impl Default for SneakyGitBehaviour {
    fn default() -> Self { Self::new() }
}

// -- SneakyGitEjectPlayerModifier ----------------------------------------------

/// Java: StepModifier<StepEjectPlayer, StepEjectPlayer.StepState>
pub struct SneakyGitEjectPlayerModifier;

impl StepModifierTrait for SneakyGitEjectPlayerModifier {
    fn applies_to(&self, step_id: StepId) -> bool { step_id == StepId::EjectPlayer }
    fn priority(&self) -> i32 { 0 }

    /// Java: SneakyGitBehaviour.handleExecuteStepHook(StepEjectPlayer step, StepState state)
    fn handle_execute_step(
        &self,
        game: &mut Game,
        _rng: &mut GameRng,
        step_state: &mut dyn std::any::Any,
    ) -> bool {
        let state = step_state
            .downcast_mut::<StepEjectPlayerHookState>()
            .expect("SneakyGitEjectPlayerModifier: step_state must be StepEjectPlayerHookState");

        let player_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => return false,
        };
        let player_state = match game.field_model.player_state(&player_id) {
            Some(ps) => ps,
            None => return false,
        };

        // Java: boolean wasCased = playerState.isCasualty()
        let was_cased = player_state.is_casualty();

        // Java: SendToBoxReason reason = SendToBoxReason.FOUL_BAN;
        let reason = if state.officious_ref {
            SendToBoxReason::OficiousRef
        } else if game.acting_player.player_id.as_deref()
            == game.original_bombardier.as_deref()
            && game.original_bombardier.is_some()
        {
            SendToBoxReason::ThrewTwoBombs
        } else {
            SendToBoxReason::FoulBan
        };

        let has_sneaky_git = game.player(&player_id)
            .map(|p| p.has_skill(SkillId::SneakyGit))
            .unwrap_or(false);

        // Java: game.getOptions().getGameOption(GameOptionId.SNEAKY_GIT_BAN_TO_KO).isEnabled()
        let sneaky_git_ban_to_ko = game.options.is_enabled("sneakyGitBanToKo");

        let turn_nr = game.turn_data().turn_nr;
        let half = game.half;
        let is_home = game.team_home.player(&player_id).is_some();

        if has_sneaky_git && sneaky_git_ban_to_ko {
            // Java: playerState.changeBase(PlayerState.KNOCKED_OUT)
            game.field_model.set_player_state(&player_id, player_state.change_base(PS_KNOCKED_OUT));
            let tr = game.game_result.team_result_mut(is_home);
            tr.player_result_mut(&player_id).send_to_box_reason = Some(reason);
            tr.player_result_mut(&player_id).send_to_box_turn = turn_nr;
            tr.player_result_mut(&player_id).send_to_box_half = half;
        } else if (state.argue_the_call_successful.is_none()
            || !state.argue_the_call_successful.unwrap_or(false))
            && !was_cased
        {
            // Java: playerState.changeBase(PlayerState.BANNED)
            game.field_model.set_player_state(&player_id, player_state.change_base(PS_BANNED));
            let tr = game.game_result.team_result_mut(is_home);
            tr.player_result_mut(&player_id).send_to_box_reason = Some(reason);
            tr.player_result_mut(&player_id).send_to_box_turn = turn_nr;
            tr.player_result_mut(&player_id).send_to_box_half = half;
        }
        // Java: return false;
        false
    }
}

// -- SneakyGitRefereeModifier --------------------------------------------------

/// Java: StepModifier<StepReferee, StepReferee.StepState>
/// The SneakyGit referee logic is already inlined in step_referee.rs.
/// This modifier exists so the skill is registered and applies_to returns true.
pub struct SneakyGitRefereeModifier;

impl StepModifierTrait for SneakyGitRefereeModifier {
    fn applies_to(&self, step_id: StepId) -> bool { step_id == StepId::Referee }
    fn priority(&self) -> i32 { 0 }
    fn handle_execute_step(
        &self,
        _game: &mut Game,
        _rng: &mut GameRng,
        _step_state: &mut dyn std::any::Any,
    ) -> bool {
        // headless: logic already inlined in step_referee.rs::execute_step()
        false
    }
}

// -- Tests --------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PS_STANDING, Rules};
    use ffb_model::model::player::Player;
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::types::FieldCoordinate;
    use crate::step::framework::test_team;

    fn make_game_with_fouler(player_id: &str, has_sneaky_git: bool) -> Game {
        let mut home = test_team("home", 0);
        home.players.push(Player {
            id: player_id.into(),
            name: "fouler".into(),
            nr: 1,
            starting_skills: if has_sneaky_git {
                vec![SkillWithValue { skill_id: SkillId::SneakyGit, value: None }]
            } else {
                vec![]
            },
            ..Default::default()
        });
        let mut game = Game::new(home, test_team("away", 0), Rules::Bb2025);
        game.acting_player.player_id = Some(player_id.into());
        game.home_playing = true;
        game.field_model.set_player_state(
            player_id,
            ffb_model::enums::PlayerState::new(PS_STANDING),
        );
        game.field_model.set_player_coordinate(player_id, FieldCoordinate::new(5, 5));
        game
    }

    #[test]
    fn register_into_adds_two_step_modifiers() {
        let mut reg = SkillRegistry::empty();
        SneakyGitBehaviour::register_into(&mut reg);
        let sb = reg.get(SkillId::SneakyGit).expect("SneakyGit must be registered");
        assert_eq!(sb.get_step_modifiers().len(), 2);
    }

    #[test]
    fn eject_player_modifier_applies_to_correct_step() {
        let m = SneakyGitEjectPlayerModifier;
        assert!(m.applies_to(StepId::EjectPlayer));
        assert!(!m.applies_to(StepId::Referee));
    }

    #[test]
    fn referee_modifier_applies_to_correct_step() {
        let m = SneakyGitRefereeModifier;
        assert!(m.applies_to(StepId::Referee));
        assert!(!m.applies_to(StepId::EjectPlayer));
    }

    #[test]
    fn no_argue_the_call_and_not_cased_sets_banned() {
        let m = SneakyGitEjectPlayerModifier;
        let mut game = make_game_with_fouler("p1", false);
        let mut hook = StepEjectPlayerHookState {
            argue_the_call_successful: None,
            officious_ref: false,
        };
        m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hook);
        let ps = game.field_model.player_state("p1").expect("player state must exist");
        assert_eq!(ps.base(), PS_BANNED);
    }

    #[test]
    fn argue_the_call_successful_prevents_ban() {
        let m = SneakyGitEjectPlayerModifier;
        let mut game = make_game_with_fouler("p1", false);
        let mut hook = StepEjectPlayerHookState {
            argue_the_call_successful: Some(true),
            officious_ref: false,
        };
        m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hook);
        let ps = game.field_model.player_state("p1").expect("player state must exist");
        assert_ne!(ps.base(), PS_BANNED);
    }

    #[test]
    fn officious_ref_reason_is_set_correctly() {
        let m = SneakyGitEjectPlayerModifier;
        let mut game = make_game_with_fouler("p1", false);
        let mut hook = StepEjectPlayerHookState {
            argue_the_call_successful: None,
            officious_ref: true,
        };
        m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hook);
        let pr = game.game_result.home.player_result("p1").expect("player result must exist");
        assert_eq!(pr.send_to_box_reason, Some(SendToBoxReason::OficiousRef));
    }

    #[test]
    fn foul_ban_reason_is_default() {
        let m = SneakyGitEjectPlayerModifier;
        let mut game = make_game_with_fouler("p1", false);
        let mut hook = StepEjectPlayerHookState::new();
        m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hook);
        let pr = game.game_result.home.player_result("p1").expect("player result must exist");
        assert_eq!(pr.send_to_box_reason, Some(SendToBoxReason::FoulBan));
    }

    #[test]
    fn threw_two_bombs_reason_when_original_bombardier() {
        let m = SneakyGitEjectPlayerModifier;
        let mut game = make_game_with_fouler("p1", false);
        game.original_bombardier = Some("p1".into());
        let mut hook = StepEjectPlayerHookState::new();
        m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hook);
        let pr = game.game_result.home.player_result("p1").expect("player result must exist");
        assert_eq!(pr.send_to_box_reason, Some(SendToBoxReason::ThrewTwoBombs));
    }

    #[test]
    fn modifier_returns_false() {
        let m = SneakyGitEjectPlayerModifier;
        let mut game = make_game_with_fouler("p1", false);
        let mut hook = StepEjectPlayerHookState::new();
        let result = m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hook);
        assert!(!result);
    }

    #[test]
    fn send_to_box_turn_and_half_recorded() {
        let m = SneakyGitEjectPlayerModifier;
        let mut game = make_game_with_fouler("p1", false);
        game.turn_data_home.turn_nr = 4;
        game.half = 2;
        let mut hook = StepEjectPlayerHookState::new();
        m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut hook);
        let pr = game.game_result.home.player_result("p1").expect("player result must exist");
        assert_eq!(pr.send_to_box_turn, 4);
        assert_eq!(pr.send_to_box_half, 2);
    }

    #[test]
    fn referee_modifier_returns_false() {
        let m = SneakyGitRefereeModifier;
        let mut game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025);
        let mut state = StepRefereeHookState::new();
        let result = m.handle_execute_step(&mut game, &mut GameRng::new(0), &mut state);
        assert!(!result);
    }

    #[test]
    fn hook_state_defaults_are_correct() {
        let h = StepEjectPlayerHookState::new();
        assert!(h.argue_the_call_successful.is_none());
        assert!(!h.officious_ref);
    }
}
