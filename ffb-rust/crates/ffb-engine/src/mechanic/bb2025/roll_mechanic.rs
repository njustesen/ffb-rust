/// 1:1 translation of com.fumbbl.ffb.server.mechanic.bb2025.RollMechanic.
use ffb_mechanics::mechanics::{
    interpret_injury_total_bb2020, requires_si_table_bb2020, serious_injury_bb2025, si_sub_type_bb2025,
    SiSubType,
};
use ffb_model::enums::{
    ApothecaryMode, PlayerAction, PlayerState, ReRollProperty, SeriousInjuryKind, TurnMode,
    PS_BADLY_HURT, PS_RIP, PS_SERIOUS_INJURY,
};
use ffb_model::model::game::Game;
use ffb_model::model::injury_attribute::InjuryAttribute;
use ffb_model::model::player::Player;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::model::turn_data::TurnData;
use ffb_model::util::rng::GameRng;
use crate::injury::InjuryContext;
use crate::mechanic::roll_mechanic::{
    injury_modifier_sum, injury_outcome_to_player_state, RollMechanic as RollMechanicTrait,
};

/// BB2025 reduction thresholds: stat cannot be reduced below this value.
/// MA→1, ST→1, AG→6, PA→6, AV→3.
fn reduction_threshold(attr: InjuryAttribute) -> i32 {
    match attr {
        InjuryAttribute::MA => 1,
        InjuryAttribute::ST => 1,
        InjuryAttribute::AG => 6,
        InjuryAttribute::PA => 6,
        InjuryAttribute::AV => 3,
        InjuryAttribute::NI => 0,
    }
}

fn can_be_reduced(attr: InjuryAttribute, current_value: i32) -> bool {
    current_value > 0 && current_value != reduction_threshold(attr)
}

fn current_stat_value(attr: InjuryAttribute, player: &Player) -> i32 {
    match attr {
        InjuryAttribute::MA => player.movement,
        InjuryAttribute::ST => player.strength,
        InjuryAttribute::AG => player.agility,
        InjuryAttribute::PA => player.passing,
        InjuryAttribute::AV => player.armour,
        InjuryAttribute::NI => 0,
    }
}

/// BB2025 SI detail table: map d6 roll to SeriousInjuryKind, with stat-floor fallback.
fn map_si_roll_bb2025(game: &Game, ctx: &InjuryContext, si_roll: i32) -> Option<SeriousInjuryKind> {
    let original = serious_injury_bb2025(si_roll)?;
    let attr = original.injury_attribute();
    let defender = ctx.defender_id.as_deref().and_then(|id| game.player(id));

    // If the stat can still be reduced, use the original injury. Otherwise SERIOUSLY_HURT.
    let reduceable = attr
        .filter(|a| *a != InjuryAttribute::NI)
        .and_then(|a| defender.map(|d| can_be_reduced(a, current_stat_value(a, d))))
        .unwrap_or(false);

    if reduceable {
        Some(original)
    } else {
        Some(SeriousInjuryKind::SeriouslyHurt)
    }
}

fn interpret_si_from_rolls(
    game: &Game,
    ctx: &InjuryContext,
    cas_roll: i32,
    si_roll: i32,
) -> Option<SeriousInjuryKind> {
    if requires_si_table_bb2020(cas_roll) {
        return map_si_roll_bb2025(game, ctx, si_roll);
    }
    si_sub_type_bb2025(cas_roll).map(|st| match st {
        SiSubType::SeriousInjury => SeriousInjuryKind::SeriousInjuryNi,
        SiSubType::SeriouslyHurt => SeriousInjuryKind::SeriouslyHurt,
    })
}

/// BB2025: d16 total (after modifiers) → PlayerState base constant.
fn map_casualty_roll_bb2025(roll: i32) -> u32 {
    if roll >= 15 { PS_RIP }
    else if roll >= 9 { PS_SERIOUS_INJURY }
    else { PS_BADLY_HURT }
}

pub struct RollMechanic;

impl RollMechanic {
    pub fn new() -> Self { Self }

    fn modes_prohibiting_re_rolls() -> &'static [TurnMode] {
        &[
            TurnMode::Kickoff,
            TurnMode::PassBlock,
            TurnMode::DumpOff,
            TurnMode::QuickSnap,
            TurnMode::BetweenTurns,
        ]
    }
}

impl Default for RollMechanic {
    fn default() -> Self { Self::new() }
}

impl RollMechanicTrait for RollMechanic {
    fn roll_casualty(&self, rng: &mut GameRng) -> [i32; 2] {
        [rng.die(16), rng.d6()]
    }

    fn interpret_injury_roll(&self, game: &Game, ctx: &mut InjuryContext) -> Option<PlayerState> {
        if game.is_finished() { return None; }
        let defender = ctx.defender_id.as_deref().and_then(|id| game.player(id));

        if defender.map(|d| d.has_skill_property(NamedProperties::PREVENT_DAMAGING_INJURY_MODIFICATIONS)).unwrap_or(false) {
            ctx.injury_modifiers.clear();
        }

        let injury_roll = match ctx.injury_roll {
            // No roll: forced injury (e.g. eaten) — return whatever is pre-set
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

        // BB2025 shares the BB2020 injury table
        interpret_injury_total_bb2020(total, is_stunty, has_thick_skull)
            .and_then(injury_outcome_to_player_state)
    }

    fn interpret_casualty_roll_and_add_modifiers(
        &self,
        _game: &Game,
        ctx: &mut InjuryContext,
        _defender: &Player,
        _is_decay_roll: bool,
    ) -> Option<PlayerState> {
        let roll = ctx.casualty_roll?;
        // TODO: CasualtyModifierFactory not yet translated; no modifiers applied
        let total = roll[0] + ctx.casualty_modifier_sum();
        Some(PlayerState::new(map_casualty_roll_bb2025(total)))
    }

    fn interpret_serious_injury_roll(
        &self,
        game: &Game,
        ctx: &InjuryContext,
    ) -> Option<SeriousInjuryKind> {
        let roll = ctx.casualty_roll?;
        let cas_roll = roll[0] + ctx.casualty_modifier_sum();
        interpret_si_from_rolls(game, ctx, cas_roll, roll[1])
    }

    fn interpret_serious_injury_roll_decay(
        &self,
        game: &Game,
        ctx: &InjuryContext,
        _use_decay: bool,
    ) -> Option<SeriousInjuryKind> {
        // BB2025: decay path delegates to normal path
        self.interpret_serious_injury_roll(game, ctx)
    }

    fn interpret_serious_injury_roll_explicit(
        &self,
        game: &Game,
        ctx: &InjuryContext,
        roll: [i32; 2],
    ) -> Option<SeriousInjuryKind> {
        let cas_roll = roll[0] + ctx.casualty_modifier_sum();
        interpret_si_from_rolls(game, ctx, cas_roll, roll[1])
    }

    fn multi_block_attacker_modifier(&self) -> i32 { -2 }
    fn multi_block_defender_modifier(&self) -> i32 { 0 }

    fn minimum_loner_roll(&self, player: &Player) -> i32 {
        player.get_skill_int_value(NamedProperties::HAS_TO_ROLL_TO_USE_TEAM_REROLL)
    }

    fn minimum_pro_roll(&self) -> i32 { 3 }

    fn allows_team_re_roll(&self, mode: TurnMode) -> bool {
        !Self::modes_prohibiting_re_rolls().contains(&mode)
    }

    fn find_additional_re_roll_property(&self, turn_data: &TurnData) -> Option<ReRollProperty> {
        if turn_data.rerolls_brilliant_coaching_one_drive > 0 {
            return Some(ReRollProperty::BrilliantCoaching);
        }
        if turn_data.rerolls_pump_up_the_crowd_one_drive > 0 {
            return Some(ReRollProperty::PumpUpTheCrowd);
        }
        if turn_data.reroll_show_star_one_drive > 0 {
            return Some(ReRollProperty::ShowStar);
        }
        None
    }

    fn is_mascot_available(&self, game: &Game, player: &Player) -> bool {
        // TODO: InducementSet stub; simplified to team-re-roll check with amount=1
        self.is_team_re_roll_available_amount(game, player, 1)
    }

    fn get_attacker_base_strength(
        &self,
        _game: &Game,
        attacker: &Player,
        _defender: &Player,
        is_multi_block: bool,
    ) -> i32 {
        let mut strength = attacker.strength_with_modifiers();
        if is_multi_block {
            strength += self.multi_block_attacker_modifier();
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

        // BB2025: addStrengthOnBlitz is applied in getTotalAttackerStrength (not base)
        let action = game.acting_player.player_action;
        let is_blitz_action = action.map(|a| a == PlayerAction::Blitz || a.is_blitz_move()).unwrap_or(false);
        if is_blitz_action && attacker.has_skill_property(NamedProperties::ADD_STRENGTH_ON_BLITZ) {
            strength += 1;
        }

        if let (Some(&attacker_coord), Some(&defender_coord)) = (
            game.field_model.player_coordinates.get(&attacker.id),
            game.field_model.player_coordinates.get(&defender.id),
        ) {
            strength = crate::util::server_util_player::ServerUtilPlayer::find_block_strength(
                game, attacker_coord, strength, defender_coord,
            );
        }

        strength
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PS_BADLY_HURT, PS_RIP, PS_SERIOUS_INJURY};
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
    fn roll_casualty_range() {
        let m = RollMechanic::new();
        let mut rng = GameRng::new(42);
        for _ in 0..20 {
            let [d16, d6] = m.roll_casualty(&mut rng);
            assert!((1..=16).contains(&d16));
            assert!((1..=6).contains(&d6));
        }
    }

    #[test]
    fn multi_block_modifiers() {
        let m = RollMechanic::new();
        assert_eq!(m.multi_block_attacker_modifier(), -2);
        assert_eq!(m.multi_block_defender_modifier(), 0);
    }

    #[test]
    fn minimum_pro_roll_is_3() {
        let m = RollMechanic::new();
        assert_eq!(m.minimum_pro_roll(), 3);
    }

    #[test]
    fn allows_re_roll_bb2025_modes() {
        let m = RollMechanic::new();
        // Prohibited
        for mode in [TurnMode::Kickoff, TurnMode::PassBlock, TurnMode::DumpOff, TurnMode::QuickSnap, TurnMode::BetweenTurns] {
            assert!(!m.allows_team_re_roll(mode), "{mode:?} should be prohibited");
        }
        // Allowed
        for mode in [TurnMode::Regular, TurnMode::Blitz] {
            assert!(m.allows_team_re_roll(mode), "{mode:?} should be allowed");
        }
    }

    #[test]
    fn find_additional_reroll_brilliant_coaching() {
        let m = RollMechanic::new();
        let mut td = TurnData::default();
        td.rerolls_brilliant_coaching_one_drive = 1;
        assert_eq!(m.find_additional_re_roll_property(&td), Some(ReRollProperty::BrilliantCoaching));
    }

    #[test]
    fn find_additional_reroll_pump_up_crowd() {
        let m = RollMechanic::new();
        let mut td = TurnData::default();
        td.rerolls_pump_up_the_crowd_one_drive = 1;
        assert_eq!(m.find_additional_re_roll_property(&td), Some(ReRollProperty::PumpUpTheCrowd));
    }

    #[test]
    fn find_additional_reroll_show_star() {
        let m = RollMechanic::new();
        let mut td = TurnData::default();
        td.reroll_show_star_one_drive = 1;
        assert_eq!(m.find_additional_re_roll_property(&td), Some(ReRollProperty::ShowStar));
    }

    #[test]
    fn find_additional_reroll_none() {
        let m = RollMechanic::new();
        let td = TurnData::default();
        assert_eq!(m.find_additional_re_roll_property(&td), None);
    }

    #[test]
    fn interpret_casualty_badly_hurt_below_9() {
        let m = RollMechanic::new();
        let g = make_game();
        let mut ctx = InjuryContext::new(ApothecaryMode::Attacker);
        ctx.casualty_roll = Some([8, 1]);
        let p = Player::default();
        let result = m.interpret_casualty_roll_and_add_modifiers(&g, &mut ctx, &p, false).unwrap();
        assert_eq!(result.base(), PS_BADLY_HURT);
    }

    #[test]
    fn interpret_casualty_serious_injury_at_9() {
        let m = RollMechanic::new();
        let g = make_game();
        let mut ctx = InjuryContext::new(ApothecaryMode::Attacker);
        ctx.casualty_roll = Some([9, 1]);
        let p = Player::default();
        let result = m.interpret_casualty_roll_and_add_modifiers(&g, &mut ctx, &p, false).unwrap();
        assert_eq!(result.base(), PS_SERIOUS_INJURY);
    }

    #[test]
    fn interpret_casualty_rip_at_15() {
        let m = RollMechanic::new();
        let g = make_game();
        let mut ctx = InjuryContext::new(ApothecaryMode::Attacker);
        ctx.casualty_roll = Some([15, 2]);
        let p = Player::default();
        let result = m.interpret_casualty_roll_and_add_modifiers(&g, &mut ctx, &p, false).unwrap();
        assert_eq!(result.base(), PS_RIP);
    }

    #[test]
    fn interpret_si_roll_seriously_hurt() {
        let m = RollMechanic::new();
        let g = make_game();
        let mut ctx = InjuryContext::new(ApothecaryMode::Attacker);
        ctx.casualty_roll = Some([9, 1]);
        assert_eq!(m.interpret_serious_injury_roll(&g, &ctx), Some(SeriousInjuryKind::SeriouslyHurt));
        ctx.casualty_roll = Some([10, 6]);
        assert_eq!(m.interpret_serious_injury_roll(&g, &ctx), Some(SeriousInjuryKind::SeriouslyHurt));
    }

    #[test]
    fn interpret_si_roll_ni() {
        let m = RollMechanic::new();
        let g = make_game();
        let mut ctx = InjuryContext::new(ApothecaryMode::Attacker);
        ctx.casualty_roll = Some([11, 1]);
        assert_eq!(m.interpret_serious_injury_roll(&g, &ctx), Some(SeriousInjuryKind::SeriousInjuryNi));
        ctx.casualty_roll = Some([12, 3]);
        assert_eq!(m.interpret_serious_injury_roll(&g, &ctx), Some(SeriousInjuryKind::SeriousInjuryNi));
    }

    #[test]
    fn interpret_si_roll_detail_table_no_defender_falls_back() {
        // No defender → can't reduce any stat → fall back to SeriouslyHurt
        let m = RollMechanic::new();
        let g = make_game();
        let mut ctx = InjuryContext::new(ApothecaryMode::Attacker);
        ctx.casualty_roll = Some([13, 1]); // d6=1 → HeadInjuryAv
        assert_eq!(m.interpret_serious_injury_roll(&g, &ctx), Some(SeriousInjuryKind::SeriouslyHurt));
    }

    #[test]
    fn interpret_injury_roll_forced_returns_preset() {
        use ffb_model::enums::PS_STUNNED;
        let m = RollMechanic::new();
        let g = make_game();
        let mut ctx = InjuryContext::new(ApothecaryMode::Attacker);
        ctx.injury = Some(PlayerState::new(PS_STUNNED));
        // injury_roll = None → forced injury
        let result = m.interpret_injury_roll(&g, &mut ctx);
        assert_eq!(result.map(|s| s.base()), Some(PS_STUNNED));
    }

    #[test]
    fn interpret_injury_roll_total_2_is_stunned() {
        use ffb_model::enums::PS_STUNNED;
        let m = RollMechanic::new();
        let g = make_game();
        let mut ctx = InjuryContext::new(ApothecaryMode::Attacker);
        ctx.injury_roll = Some([1, 1]); // total 2 → stunned
        let result = m.interpret_injury_roll(&g, &mut ctx).unwrap();
        assert_eq!(result.base(), PS_STUNNED);
    }

    #[test]
    fn interpret_injury_roll_total_10_is_casualty() {
        let m = RollMechanic::new();
        let g = make_game();
        let mut ctx = InjuryContext::new(ApothecaryMode::Attacker);
        ctx.injury_roll = Some([5, 5]); // total 10 → casualty → None
        assert!(m.interpret_injury_roll(&g, &mut ctx).is_none());
    }
}
