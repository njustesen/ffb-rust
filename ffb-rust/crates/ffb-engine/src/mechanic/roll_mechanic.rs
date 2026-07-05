/// 1:1 translation of com.fumbbl.ffb.server.mechanic.RollMechanic (abstract base).
///
/// Java abstract class → Rust trait. Concrete edition implementations:
///   bb2016::RollMechanic, bb2020::RollMechanic, bb2025::RollMechanic.
///
/// Deferred: use_re_roll, ask_for_re_roll_if_available (require IStep / dialog infra).
use ffb_mechanics::mechanics::InjuryOutcome;
use ffb_model::enums::{
    PlayerState, ReRollProperty, SeriousInjuryKind, SkillId, TurnMode,
    PS_BADLY_HURT, PS_KNOCKED_OUT, PS_STUNNED,
};
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::model::turn_data::TurnData;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::rng::GameRng;
use crate::injury::InjuryContext;

pub trait RollMechanic: Send + Sync {
    // ── Abstract (edition-specific) ───────────────────────────────────────────

    /// Returns [primary_die, secondary_die].
    /// BB2016: [d6, d8]. BB2020/BB2025: [d16, d6].
    fn roll_casualty(&self, rng: &mut GameRng) -> [i32; 2];

    /// Interprets the injury roll, returning the PlayerState or None (= casualty).
    fn interpret_injury_roll(&self, game: &Game, ctx: &mut InjuryContext) -> Option<PlayerState>;

    /// Interprets the casualty roll, adds modifiers to ctx, returns PlayerState tier.
    fn interpret_casualty_roll_and_add_modifiers(
        &self,
        game: &Game,
        ctx: &mut InjuryContext,
        defender: &Player,
        is_decay_roll: bool,
    ) -> Option<PlayerState>;

    /// Interprets the SI detail roll from ctx (uses ctx.casualty_roll).
    fn interpret_serious_injury_roll(
        &self,
        game: &Game,
        ctx: &InjuryContext,
    ) -> Option<SeriousInjuryKind>;

    /// Variant with explicit use_decay flag (BB2016 Decay skill path).
    fn interpret_serious_injury_roll_decay(
        &self,
        game: &Game,
        ctx: &InjuryContext,
        use_decay: bool,
    ) -> Option<SeriousInjuryKind>;

    /// Variant with explicit roll array.
    fn interpret_serious_injury_roll_explicit(
        &self,
        game: &Game,
        ctx: &InjuryContext,
        roll: [i32; 2],
    ) -> Option<SeriousInjuryKind>;

    fn multi_block_attacker_modifier(&self) -> i32;
    fn multi_block_defender_modifier(&self) -> i32;
    fn minimum_loner_roll(&self, player: &Player) -> i32;
    fn minimum_pro_roll(&self) -> i32;
    fn allows_team_re_roll(&self, mode: TurnMode) -> bool;
    fn find_additional_re_roll_property(&self, turn_data: &TurnData) -> Option<ReRollProperty>;
    fn is_mascot_available(&self, game: &Game, player: &Player) -> bool;

    fn get_total_attacker_strength(
        &self,
        game: &Game,
        attacker: &Player,
        defender: &Player,
        is_multi_block: bool,
        successful_dauntless: bool,
        double_target_strength: bool,
        defender_strength: i32,
    ) -> i32;

    fn get_attacker_base_strength(
        &self,
        game: &Game,
        attacker: &Player,
        defender: &Player,
        is_multi_block: bool,
    ) -> i32;

    // ── Concrete (shared across all editions) ─────────────────────────────────

    /// Java: isProReRollAvailable — player has Pro and hasn't used it yet.
    fn is_pro_re_roll_available(&self, _game: &Game, player: &Player) -> bool {
        player.has_skill_property(NamedProperties::CAN_REROLL_ONCE_PER_TURN)
            && !player.used_skills.contains(&SkillId::Pro)
    }

    /// Java: isSingleUseReRollAvailable — single-use re-rolls remain on TurnData.
    fn is_single_use_re_roll_available(&self, game: &Game, player: &Player) -> bool {
        self.is_team_re_roll_available_amount(game, player, game.turn_data().single_use_rerolls)
    }

    /// Java: isTeamReRollAvailable — team has TRR and mode allows it.
    fn is_team_re_roll_available(&self, game: &Game, player: &Player) -> bool {
        self.is_team_re_roll_available_amount(game, player, game.turn_data().rerolls)
    }

    /// Java: isTeamReRollAvailable(amount) guarded form.
    fn is_team_re_roll_available_amount(&self, game: &Game, player: &Player, amount: i32) -> bool {
        let td = game.turn_data();
        if td.reroll_used { return false; }
        if amount <= 0 { return false; }
        if !self.allows_team_re_roll(game.turn_mode) { return false; }
        if !game.active_team().has_player(&player.id) { return false; }
        let home_has = game.team_home.has_player(&player.id);
        let away_has = game.team_away.has_player(&player.id);
        let mode = game.turn_mode;
        if (mode == TurnMode::BombHome || mode == TurnMode::BombHomeBlitz) && !home_has {
            return false;
        }
        if (mode == TurnMode::BombAway || mode == TurnMode::BombAwayBlitz) && !away_has {
            return false;
        }
        true
    }
}

/// Convert `InjuryOutcome` to `PlayerState` (None = casualty).
pub fn injury_outcome_to_player_state(outcome: InjuryOutcome) -> Option<PlayerState> {
    match outcome {
        InjuryOutcome::Stunned    => Some(PlayerState::new(PS_STUNNED)),
        InjuryOutcome::KnockedOut => Some(PlayerState::new(PS_KNOCKED_OUT)),
        InjuryOutcome::BadlyHurt  => Some(PlayerState::new(PS_BADLY_HURT)),
        InjuryOutcome::Casualty   => None,
    }
}

/// Sum of injury modifier values stored in `ctx`.
pub fn injury_modifier_sum(ctx: &InjuryContext) -> i32 {
    ctx.injury_modifiers.iter().map(|m| m.value).sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::game::Game;
    use ffb_model::model::player::Player;

    fn make_game() -> Game {
        Game::new(
            crate::step::framework::test_team("home", 0),
            crate::step::framework::test_team("away", 0),
            Rules::Bb2025,
        )
    }

    struct TestMechanic;

    impl RollMechanic for TestMechanic {
        fn roll_casualty(&self, rng: &mut GameRng) -> [i32; 2] { [rng.die(16), rng.d6()] }
        fn interpret_injury_roll(&self, _g: &Game, _c: &mut InjuryContext) -> Option<PlayerState> { None }
        fn interpret_casualty_roll_and_add_modifiers(&self, _g: &Game, _c: &mut InjuryContext, _d: &Player, _dec: bool) -> Option<PlayerState> { None }
        fn interpret_serious_injury_roll(&self, _g: &Game, _c: &InjuryContext) -> Option<SeriousInjuryKind> { None }
        fn interpret_serious_injury_roll_decay(&self, _g: &Game, _c: &InjuryContext, _u: bool) -> Option<SeriousInjuryKind> { None }
        fn interpret_serious_injury_roll_explicit(&self, _g: &Game, _c: &InjuryContext, _r: [i32; 2]) -> Option<SeriousInjuryKind> { None }
        fn multi_block_attacker_modifier(&self) -> i32 { -2 }
        fn multi_block_defender_modifier(&self) -> i32 { 0 }
        fn minimum_loner_roll(&self, _p: &Player) -> i32 { 4 }
        fn minimum_pro_roll(&self) -> i32 { 3 }
        fn allows_team_re_roll(&self, mode: TurnMode) -> bool {
            !matches!(mode, TurnMode::Kickoff | TurnMode::PassBlock | TurnMode::DumpOff)
        }
        fn find_additional_re_roll_property(&self, _td: &TurnData) -> Option<ReRollProperty> { None }
        fn is_mascot_available(&self, _g: &Game, _p: &Player) -> bool { false }
        fn get_total_attacker_strength(&self, _g: &Game, _a: &Player, _d: &Player, _mb: bool, _sd: bool, _dt: bool, _ds: i32) -> i32 { 1 }
        fn get_attacker_base_strength(&self, _g: &Game, _a: &Player, _d: &Player, _mb: bool) -> i32 { 1 }
    }

    #[test]
    fn allows_team_re_roll_blocks_kickoff() {
        let m = TestMechanic;
        assert!(!m.allows_team_re_roll(TurnMode::Kickoff));
        assert!(m.allows_team_re_roll(TurnMode::Regular));
        assert!(m.allows_team_re_roll(TurnMode::Blitz));
    }

    #[test]
    fn is_pro_re_roll_without_skill() {
        let m = TestMechanic;
        let g = make_game();
        let p = Player::default();
        assert!(!m.is_pro_re_roll_available(&g, &p));
    }

    #[test]
    fn is_team_re_roll_no_rerolls() {
        let m = TestMechanic;
        let g = make_game();
        let p = Player::default();
        assert!(!m.is_team_re_roll_available(&g, &p));
    }

    #[test]
    fn injury_outcome_stunned_maps_to_ps_stunned() {
        let ps = injury_outcome_to_player_state(InjuryOutcome::Stunned).unwrap();
        assert_eq!(ps.base(), PS_STUNNED);
    }

    #[test]
    fn injury_outcome_casualty_maps_to_none() {
        assert!(injury_outcome_to_player_state(InjuryOutcome::Casualty).is_none());
    }
}
