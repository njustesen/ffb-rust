/// 1:1 translation of com.fumbbl.ffb.server.mechanic.bb2020.RollMechanic.
use ffb_mechanics::mechanics::{
    casualty_tier_bb2020, interpret_injury_total_bb2020, requires_si_table_bb2020,
    serious_injury_bb2020, si_sub_type_bb2020, CasualtyTier, SiSubType,
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

/// BB2020 stat reduction thresholds (same values as BB2025).
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

/// BB2020 ordered SI injuries indexed by d6 roll (0-based: `roll - 1`).
const ORDERED_INJURIES: [SeriousInjuryKind; 6] = [
    SeriousInjuryKind::HeadInjuryAv,       // d6=1
    SeriousInjuryKind::HeadInjuryAv,       // d6=2
    SeriousInjuryKind::SmashedKneeMa,      // d6=3
    SeriousInjuryKind::BrokenArmPa,        // d6=4
    SeriousInjuryKind::NeckInjuryAg,       // d6=5
    SeriousInjuryKind::DislocatedShoulderSt, // d6=6
];

/// BB2020 `mapSIRoll`: returns a reduceable injury for this player, or the original.
/// Java shuffles the reduceable list; Rust returns the first reduceable (no GameRng available here).
fn map_si_roll_bb2020(game: &Game, ctx: &InjuryContext, si_roll: i32) -> Option<SeriousInjuryKind> {
    let original = serious_injury_bb2020(si_roll)?;
    let defender = ctx.defender_id.as_deref().and_then(|id| game.player(id));

    let reduceable: Vec<SeriousInjuryKind> = ORDERED_INJURIES.iter().copied()
        .filter(|&inj| {
            inj.injury_attribute()
                .filter(|a| *a != InjuryAttribute::NI)
                .map(|a| defender
                    .map(|d| can_be_reduced(a, current_stat_value(a, d)))
                    .unwrap_or(false))
                .unwrap_or(false)
        })
        .collect();

    if reduceable.is_empty() || reduceable.contains(&original) {
        Some(original)
    } else {
        // Java: Collections.shuffle(reduceable); return reduceable.get(0)
        // No GameRng here — return first reduceable deterministically
        Some(reduceable[0])
    }
}

fn interpret_si_from_rolls_bb2020(
    game: &Game,
    ctx: &InjuryContext,
    cas_roll: i32,
    si_roll: i32,
) -> Option<SeriousInjuryKind> {
    if requires_si_table_bb2020(cas_roll) {
        return map_si_roll_bb2020(game, ctx, si_roll);
    }
    si_sub_type_bb2020(cas_roll).map(|st| match st {
        SiSubType::SeriousInjury => SeriousInjuryKind::SeriousInjuryNi,
        SiSubType::SeriouslyHurt => SeriousInjuryKind::SeriouslyHurt,
    })
}

pub struct RollMechanic;

impl RollMechanic {
    pub fn new() -> Self { Self }

    fn modes_prohibiting_re_rolls() -> &'static [TurnMode] {
        &[
            TurnMode::Kickoff,
            TurnMode::PassBlock,
            TurnMode::DumpOff,
            TurnMode::Blitz,
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

        interpret_injury_total_bb2020(total, is_stunty, has_thick_skull)
            .and_then(injury_outcome_to_player_state)
    }

    fn interpret_casualty_roll_and_add_modifiers(
        &self,
        _game: &Game,
        ctx: &mut InjuryContext,
        defender: &Player,
        _is_decay_roll: bool,
    ) -> Option<PlayerState> {
        let roll = ctx.casualty_roll?;
        let modifiers = ffb_mechanics::modifiers::CasualtyModifierFactory::new().find_modifiers(defender);
        ctx.add_casualty_modifiers(modifiers);
        let total = roll[0] + ctx.casualty_modifier_sum();
        let tier = casualty_tier_bb2020(total);
        let ps = match tier {
            CasualtyTier::Dead          => PS_RIP,
            CasualtyTier::SeriousInjury => PS_SERIOUS_INJURY,
            CasualtyTier::BadlyHurt     => PS_BADLY_HURT,
        };
        Some(PlayerState::new(ps))
    }

    fn interpret_serious_injury_roll(
        &self,
        game: &Game,
        ctx: &InjuryContext,
    ) -> Option<SeriousInjuryKind> {
        let roll = ctx.casualty_roll?;
        let cas_roll = roll[0] + ctx.casualty_modifier_sum();
        interpret_si_from_rolls_bb2020(game, ctx, cas_roll, roll[1])
    }

    fn interpret_serious_injury_roll_decay(
        &self,
        game: &Game,
        ctx: &InjuryContext,
        _use_decay: bool,
    ) -> Option<SeriousInjuryKind> {
        // BB2020: useDecay delegates to normal path
        self.interpret_serious_injury_roll(game, ctx)
    }

    fn interpret_serious_injury_roll_explicit(
        &self,
        game: &Game,
        ctx: &InjuryContext,
        roll: [i32; 2],
    ) -> Option<SeriousInjuryKind> {
        let cas_roll = roll[0] + ctx.casualty_modifier_sum();
        interpret_si_from_rolls_bb2020(game, ctx, cas_roll, roll[1])
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

    fn find_additional_re_roll_property(&self, _turn_data: &TurnData) -> Option<ReRollProperty> {
        None
    }

    fn is_mascot_available(&self, _game: &Game, _player: &Player) -> bool {
        false
    }

    /// BB2020: weakenOpposingBlitzer and addStrengthOnBlitz in base strength (same as BB2016).
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
    fn roll_casualty_bb2020_range() {
        let m = RollMechanic::new();
        let mut rng = GameRng::new(42);
        for _ in 0..20 {
            let [d16, d6] = m.roll_casualty(&mut rng);
            assert!((1..=16).contains(&d16), "d16={d16}");
            assert!((1..=6).contains(&d6), "d6={d6}");
        }
    }

    #[test]
    fn multi_block_modifiers_bb2020() {
        let m = RollMechanic::new();
        assert_eq!(m.multi_block_attacker_modifier(), -2);
        assert_eq!(m.multi_block_defender_modifier(), 0);
    }

    #[test]
    fn minimum_pro_roll_is_3_bb2020() {
        let m = RollMechanic::new();
        assert_eq!(m.minimum_pro_roll(), 3);
    }

    #[test]
    fn allows_re_roll_bb2020_prohibited() {
        let m = RollMechanic::new();
        for mode in [TurnMode::Kickoff, TurnMode::PassBlock, TurnMode::DumpOff, TurnMode::Blitz, TurnMode::QuickSnap, TurnMode::BetweenTurns] {
            assert!(!m.allows_team_re_roll(mode), "{mode:?} should be prohibited");
        }
    }

    #[test]
    fn allows_re_roll_bb2020_allowed() {
        let m = RollMechanic::new();
        assert!(m.allows_team_re_roll(TurnMode::Regular));
    }

    #[test]
    fn no_additional_reroll_bb2020() {
        let m = RollMechanic::new();
        let td = TurnData::default();
        assert_eq!(m.find_additional_re_roll_property(&td), None);
    }

    #[test]
    fn casualty_badly_hurt_bb2020() {
        let m = RollMechanic::new();
        let g = make_game();
        let mut ctx = InjuryContext::new(ApothecaryMode::Attacker);
        let p = Player::default();
        ctx.casualty_roll = Some([6, 1]); // 6 < 7 → BH
        let ps = m.interpret_casualty_roll_and_add_modifiers(&g, &mut ctx, &p, false).unwrap();
        assert_eq!(ps.base(), PS_BADLY_HURT);
    }

    #[test]
    fn casualty_serious_injury_at_7_bb2020() {
        let m = RollMechanic::new();
        let g = make_game();
        let mut ctx = InjuryContext::new(ApothecaryMode::Attacker);
        let p = Player::default();
        ctx.casualty_roll = Some([7, 1]);
        let ps = m.interpret_casualty_roll_and_add_modifiers(&g, &mut ctx, &p, false).unwrap();
        assert_eq!(ps.base(), PS_SERIOUS_INJURY);
    }

    #[test]
    fn casualty_rip_at_15_bb2020() {
        let m = RollMechanic::new();
        let g = make_game();
        let mut ctx = InjuryContext::new(ApothecaryMode::Attacker);
        let p = Player::default();
        ctx.casualty_roll = Some([15, 2]);
        let ps = m.interpret_casualty_roll_and_add_modifiers(&g, &mut ctx, &p, false).unwrap();
        assert_eq!(ps.base(), PS_RIP);
    }

    #[test]
    fn si_roll_seriously_hurt_7_to_9_bb2020() {
        let m = RollMechanic::new();
        let g = make_game();
        let mut ctx = InjuryContext::new(ApothecaryMode::Attacker);
        for cas in [7i32, 8, 9] {
            ctx.casualty_roll = Some([cas, 1]);
            assert_eq!(m.interpret_serious_injury_roll(&g, &ctx), Some(SeriousInjuryKind::SeriouslyHurt), "cas={cas}");
        }
    }

    #[test]
    fn si_roll_serious_injury_ni_10_to_12_bb2020() {
        let m = RollMechanic::new();
        let g = make_game();
        let mut ctx = InjuryContext::new(ApothecaryMode::Attacker);
        for cas in [10i32, 11, 12] {
            ctx.casualty_roll = Some([cas, 1]);
            assert_eq!(m.interpret_serious_injury_roll(&g, &ctx), Some(SeriousInjuryKind::SeriousInjuryNi), "cas={cas}");
        }
    }

    #[test]
    fn si_roll_detail_table_d6_1_is_head_injury_bb2020() {
        let m = RollMechanic::new();
        let g = make_game();
        let mut ctx = InjuryContext::new(ApothecaryMode::Attacker);
        // cas=13 → detail table, si_roll=1 → HeadInjuryAv
        ctx.casualty_roll = Some([13, 1]);
        // No defender → reduceable filter yields nothing → return original
        assert_eq!(m.interpret_serious_injury_roll(&g, &ctx), Some(SeriousInjuryKind::HeadInjuryAv));
    }

    #[test]
    fn si_roll_detail_table_d6_6_is_dislocated_shoulder_bb2020() {
        let m = RollMechanic::new();
        let g = make_game();
        let mut ctx = InjuryContext::new(ApothecaryMode::Attacker);
        ctx.casualty_roll = Some([14, 6]);
        assert_eq!(m.interpret_serious_injury_roll(&g, &ctx), Some(SeriousInjuryKind::DislocatedShoulderSt));
    }

    #[test]
    fn injury_roll_forced_returns_preset_bb2020() {
        let m = RollMechanic::new();
        let g = make_game();
        let mut ctx = InjuryContext::new(ApothecaryMode::Attacker);
        ctx.injury = Some(PlayerState::new(PS_STUNNED));
        assert_eq!(m.interpret_injury_roll(&g, &mut ctx).map(|s| s.base()), Some(PS_STUNNED));
    }

    #[test]
    fn injury_roll_total_2_stunned_bb2020() {
        let m = RollMechanic::new();
        let g = make_game();
        let mut ctx = InjuryContext::new(ApothecaryMode::Attacker);
        ctx.injury_roll = Some([1, 1]);
        assert_eq!(m.interpret_injury_roll(&g, &mut ctx).map(|s| s.base()), Some(PS_STUNNED));
    }

    #[test]
    fn injury_roll_total_10_casualty_bb2020() {
        let m = RollMechanic::new();
        let g = make_game();
        let mut ctx = InjuryContext::new(ApothecaryMode::Attacker);
        ctx.injury_roll = Some([5, 5]);
        assert!(m.interpret_injury_roll(&g, &mut ctx).is_none());
    }
}
