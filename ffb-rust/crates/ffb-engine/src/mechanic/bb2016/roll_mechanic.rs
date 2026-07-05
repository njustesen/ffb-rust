/// 1:1 translation of com.fumbbl.ffb.server.mechanic.bb2016.RollMechanic.
use ffb_mechanics::mechanics::{
    casualty_tier_bb2016, interpret_injury_total_bb2016, requires_si_table_bb2016,
    serious_injury_bb2016, CasualtyTier,
};
use ffb_model::enums::{
    ApothecaryMode, PlayerAction, PlayerState, ReRollProperty, SeriousInjuryKind, TurnMode,
    PS_BADLY_HURT, PS_RIP, PS_SERIOUS_INJURY,
};
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::model::turn_data::TurnData;
use ffb_model::util::rng::GameRng;
use crate::injury::InjuryContext;
use crate::mechanic::roll_mechanic::{
    injury_modifier_sum, injury_outcome_to_player_state, RollMechanic as RollMechanicTrait,
};

pub struct RollMechanic;

impl RollMechanic {
    pub fn new() -> Self { Self }

    fn modes_prohibiting_re_rolls() -> &'static [TurnMode] {
        &[TurnMode::Kickoff, TurnMode::PassBlock, TurnMode::DumpOff]
    }
}

impl Default for RollMechanic {
    fn default() -> Self { Self::new() }
}

impl RollMechanicTrait for RollMechanic {
    /// BB2016 uses d6 + d8 (old two-die casualty table).
    fn roll_casualty(&self, rng: &mut GameRng) -> [i32; 2] {
        [rng.d6(), rng.d8()]
    }

    fn interpret_injury_roll(&self, game: &Game, ctx: &mut InjuryContext) -> Option<PlayerState> {
        if game.is_finished() { return None; }
        let defender = ctx.defender_id.as_deref().and_then(|id| game.player(id));

        if defender.map(|d| d.has_skill_property(NamedProperties::PREVENT_DAMAGING_INJURY_MODIFICATIONS)).unwrap_or(false) {
            ctx.injury_modifiers.clear();
        }

        let injury_roll = match ctx.injury_roll {
            None => return ctx.injury,
            Some(r) => r,
        };

        let is_stunty = defender
            .map(|d| d.has_skill_property(NamedProperties::IS_HURT_MORE_EASILY))
            .unwrap_or(false);
        let has_thick_skull = defender
            .map(|d| d.has_skill_property(NamedProperties::CONVERT_KO_TO_STUN_ON_8))
            .unwrap_or(false);

        let total = injury_roll[0] + injury_roll[1] + injury_modifier_sum(ctx);

        interpret_injury_total_bb2016(total, is_stunty, has_thick_skull)
            .and_then(injury_outcome_to_player_state)
    }

    fn interpret_casualty_roll_and_add_modifiers(
        &self,
        _game: &Game,
        ctx: &mut InjuryContext,
        _defender: &Player,
        is_decay_roll: bool,
    ) -> Option<PlayerState> {
        let roll = if is_decay_roll { ctx.casualty_roll_decay } else { ctx.casualty_roll };
        let roll = roll?;
        let tier = casualty_tier_bb2016(roll[0]);
        let ps = match tier {
            CasualtyTier::Dead         => PS_RIP,
            CasualtyTier::SeriousInjury => PS_SERIOUS_INJURY,
            CasualtyTier::BadlyHurt    => PS_BADLY_HURT,
        };
        Some(PlayerState::new(ps))
    }

    fn interpret_serious_injury_roll(
        &self,
        _game: &Game,
        ctx: &InjuryContext,
    ) -> Option<SeriousInjuryKind> {
        self.interpret_serious_injury_roll_decay(_game, ctx, false)
    }

    fn interpret_serious_injury_roll_decay(
        &self,
        _game: &Game,
        ctx: &InjuryContext,
        use_decay: bool,
    ) -> Option<SeriousInjuryKind> {
        let roll = if use_decay { ctx.casualty_roll_decay } else { ctx.casualty_roll };
        let roll = roll?;
        if requires_si_table_bb2016(roll[0]) {
            serious_injury_bb2016(roll[0], roll[1])
        } else {
            None
        }
    }

    fn interpret_serious_injury_roll_explicit(
        &self,
        _game: &Game,
        _ctx: &InjuryContext,
        roll: [i32; 2],
    ) -> Option<SeriousInjuryKind> {
        if requires_si_table_bb2016(roll[0]) {
            serious_injury_bb2016(roll[0], roll[1])
        } else {
            None
        }
    }

    fn multi_block_attacker_modifier(&self) -> i32 { 0 }
    fn multi_block_defender_modifier(&self) -> i32 { 2 }

    fn minimum_loner_roll(&self, _player: &Player) -> i32 { 4 }
    fn minimum_pro_roll(&self) -> i32 { 4 }

    fn allows_team_re_roll(&self, mode: TurnMode) -> bool {
        !Self::modes_prohibiting_re_rolls().contains(&mode)
    }

    fn find_additional_re_roll_property(&self, _turn_data: &TurnData) -> Option<ReRollProperty> {
        None
    }

    fn is_mascot_available(&self, _game: &Game, _player: &Player) -> bool {
        false
    }

    /// BB2016: addStrengthOnBlitz and weakenOpposingBlitzer applied in base strength.
    fn get_attacker_base_strength(
        &self,
        game: &Game,
        attacker: &Player,
        defender: &Player,
        is_multi_block: bool,
    ) -> i32 {
        let mut strength = attacker.strength_with_modifiers();
        if is_multi_block {
            strength += self.multi_block_attacker_modifier();
        }
        let ap = &game.acting_player;
        let action = ap.player_action;
        let is_blitz = action.map(|a| a == PlayerAction::Blitz || a.is_blitz_move()).unwrap_or(false);
        if is_blitz && ap.has_moved && defender.has_skill_property(NamedProperties::WEAKEN_OPPOSING_BLITZER) {
            strength -= 1;
        }
        if is_blitz {
            if let Some(ap_player) = ap.player_id.as_deref().and_then(|id| game.player(id)) {
                if ap_player.has_skill_property(NamedProperties::ADD_STRENGTH_ON_BLITZ) {
                    strength += 1;
                }
            }
        }
        strength.max(1)
    }

    fn get_total_attacker_strength(
        &self,
        game: &Game,
        attacker: &Player,
        defender: &Player,
        is_multi_block: bool,
        successful_dauntless: bool,
        double_target_strength: bool,
        defender_strength: i32,
    ) -> i32 {
        let mut strength = self.get_attacker_base_strength(game, attacker, defender, is_multi_block);
        if successful_dauntless {
            let target = if double_target_strength { 2 * defender_strength } else { defender_strength };
            strength = strength.max(target);
        }
        if let (Some(&atk_coord), Some(&def_coord)) = (
            game.field_model.player_coordinates.get(&attacker.id),
            game.field_model.player_coordinates.get(&defender.id),
        ) {
            strength = crate::util::server_util_player::ServerUtilPlayer::find_block_strength(
                game, atk_coord, strength, def_coord,
            );
        }
        strength
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PS_BADLY_HURT, PS_RIP, PS_SERIOUS_INJURY, PS_STUNNED};
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

    #[test]
    fn roll_casualty_produces_d6_and_d8() {
        let m = RollMechanic::new();
        let mut rng = GameRng::new(7);
        for _ in 0..20 {
            let [d6, d8] = m.roll_casualty(&mut rng);
            assert!((1..=6).contains(&d6), "d6={d6}");
            assert!((1..=8).contains(&d8), "d8={d8}");
        }
    }

    #[test]
    fn multi_block_modifiers_bb2016() {
        let m = RollMechanic::new();
        assert_eq!(m.multi_block_attacker_modifier(), 0);
        assert_eq!(m.multi_block_defender_modifier(), 2);
    }

    #[test]
    fn minimum_rolls_bb2016() {
        let m = RollMechanic::new();
        let p = Player::default();
        assert_eq!(m.minimum_loner_roll(&p), 4);
        assert_eq!(m.minimum_pro_roll(), 4);
    }

    #[test]
    fn allows_re_roll_bb2016_modes() {
        let m = RollMechanic::new();
        // Prohibited
        for mode in [TurnMode::Kickoff, TurnMode::PassBlock, TurnMode::DumpOff] {
            assert!(!m.allows_team_re_roll(mode), "{mode:?} should be prohibited");
        }
        // Allowed (Blitz and QuickSnap are NOT prohibited in BB2016)
        for mode in [TurnMode::Regular, TurnMode::Blitz, TurnMode::QuickSnap] {
            assert!(m.allows_team_re_roll(mode), "{mode:?} should be allowed");
        }
    }

    #[test]
    fn no_additional_reroll_property_bb2016() {
        let m = RollMechanic::new();
        let td = TurnData::default();
        assert_eq!(m.find_additional_re_roll_property(&td), None);
    }

    #[test]
    fn mascot_unavailable_bb2016() {
        let m = RollMechanic::new();
        let g = make_game();
        let p = Player::default();
        assert!(!m.is_mascot_available(&g, &p));
    }

    #[test]
    fn casualty_badly_hurt_bb2016() {
        let m = RollMechanic::new();
        let g = make_game();
        let mut ctx = InjuryContext::new(ApothecaryMode::Attacker);
        let p = Player::default();
        for die1 in [1, 2, 3] {
            ctx.casualty_roll = Some([die1, 1]);
            let ps = m.interpret_casualty_roll_and_add_modifiers(&g, &mut ctx, &p, false).unwrap();
            assert_eq!(ps.base(), PS_BADLY_HURT, "die1={die1}");
        }
    }

    #[test]
    fn casualty_serious_injury_bb2016() {
        let m = RollMechanic::new();
        let g = make_game();
        let mut ctx = InjuryContext::new(ApothecaryMode::Attacker);
        let p = Player::default();
        for die1 in [4, 5] {
            ctx.casualty_roll = Some([die1, 1]);
            let ps = m.interpret_casualty_roll_and_add_modifiers(&g, &mut ctx, &p, false).unwrap();
            assert_eq!(ps.base(), PS_SERIOUS_INJURY, "die1={die1}");
        }
    }

    #[test]
    fn casualty_rip_bb2016() {
        let m = RollMechanic::new();
        let g = make_game();
        let mut ctx = InjuryContext::new(ApothecaryMode::Attacker);
        let p = Player::default();
        ctx.casualty_roll = Some([6, 1]);
        let ps = m.interpret_casualty_roll_and_add_modifiers(&g, &mut ctx, &p, false).unwrap();
        assert_eq!(ps.base(), PS_RIP);
    }

    #[test]
    fn casualty_decay_roll_bb2016() {
        let m = RollMechanic::new();
        let g = make_game();
        let mut ctx = InjuryContext::new(ApothecaryMode::Attacker);
        let p = Player::default();
        ctx.casualty_roll_decay = Some([6, 3]);
        let ps = m.interpret_casualty_roll_and_add_modifiers(&g, &mut ctx, &p, true).unwrap();
        assert_eq!(ps.base(), PS_RIP);
    }

    #[test]
    fn si_roll_serious_injury_d6_4_d8_1() {
        let m = RollMechanic::new();
        let g = make_game();
        let ctx = InjuryContext::new(ApothecaryMode::Attacker);
        // die1=4, die2=1 → BROKEN_RIBS
        let mut ctx2 = ctx;
        ctx2.casualty_roll = Some([4, 1]);
        assert_eq!(m.interpret_serious_injury_roll(&g, &ctx2), Some(SeriousInjuryKind::BrokenRibs));
    }

    #[test]
    fn si_roll_serious_injury_d6_5_d8_2() {
        let m = RollMechanic::new();
        let g = make_game();
        let mut ctx = InjuryContext::new(ApothecaryMode::Attacker);
        // die1=5, die2=2 → SMASHED_KNEE (NI)
        ctx.casualty_roll = Some([5, 2]);
        assert_eq!(m.interpret_serious_injury_roll(&g, &ctx), Some(SeriousInjuryKind::SmashedKneeB2016));
    }

    #[test]
    fn si_roll_no_si_for_die1_6() {
        let m = RollMechanic::new();
        let g = make_game();
        let mut ctx = InjuryContext::new(ApothecaryMode::Attacker);
        ctx.casualty_roll = Some([6, 1]); // die1=6 → dead, not SI table
        assert_eq!(m.interpret_serious_injury_roll(&g, &ctx), None);
    }

    #[test]
    fn injury_roll_forced_bb2016() {
        use ffb_model::enums::PS_STUNNED;
        let m = RollMechanic::new();
        let g = make_game();
        let mut ctx = InjuryContext::new(ApothecaryMode::Attacker);
        ctx.injury = Some(PlayerState::new(PS_STUNNED));
        let result = m.interpret_injury_roll(&g, &mut ctx);
        assert_eq!(result.map(|s| s.base()), Some(PS_STUNNED));
    }

    #[test]
    fn injury_roll_total_2_stunned_bb2016() {
        let m = RollMechanic::new();
        let g = make_game();
        let mut ctx = InjuryContext::new(ApothecaryMode::Attacker);
        ctx.injury_roll = Some([1, 1]);
        assert_eq!(m.interpret_injury_roll(&g, &mut ctx).map(|s| s.base()), Some(PS_STUNNED));
    }

    #[test]
    fn injury_roll_total_10_is_casualty_bb2016() {
        let m = RollMechanic::new();
        let g = make_game();
        let mut ctx = InjuryContext::new(ApothecaryMode::Attacker);
        ctx.injury_roll = Some([5, 5]);
        assert!(m.interpret_injury_roll(&g, &mut ctx).is_none());
    }
}
