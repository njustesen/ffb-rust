/// Translation of com.fumbbl.ffb.server.injury.modification.OldProModification.
///
/// Re-rolls a single armour die. Gate: Pro re-roll is available, armour roll exists,
/// and (armorBroken == selfInflicted OR spottedFoul).
/// requiresConditionalReRollSkill = true.
use ffb_model::enums::ApothecaryMode;
use ffb_model::model::game::Game;
use ffb_model::model::SkillUse;
use ffb_model::util::rng::GameRng;
use crate::injury::InjuryContext;
use crate::injury::modification::{InjuryContextModification, ModificationParams};
use crate::injury::modification::old_pro_modification_params::OldProModificationParams;
use crate::util::util_server_re_roll::UtilServerReRoll;

pub struct OldProModification {
    skill_id: Option<u16>,
}

impl OldProModification {
    pub fn new() -> Self { Self { skill_id: None } }

    fn is_spotted_foul(ctx: &InjuryContext, injury_type_name: &str) -> bool {
        if let Some([d1, d2]) = ctx.armor_roll {
            let is_foul = matches!(injury_type_name, "Foul" | "FoulForSpp" | "FoulWithChainsaw" | "FoulForSppWithChainsaw");
            d1 == d2 && is_foul
        } else {
            false
        }
    }

    fn is_self_inflicted(game: &Game, ctx: &InjuryContext, injury_type_name: &str) -> bool {
        let attacker = ctx.attacker_id.as_deref().and_then(|id| game.player(id));
        let defender = ctx.defender_id.as_deref().and_then(|id| game.player(id));

        let hurt_team_mate = match (attacker, defender) {
            (Some(a), Some(d)) => {
                game.player_team_id(&a.id) == game.player_team_id(&d.id)
                    && ctx.apothecary_mode != ApothecaryMode::AnimalSavagery
            }
            _ => false,
        };

        let no_attacker = ctx.attacker_id.as_deref().map(|s| s.is_empty()).unwrap_or(true);
        let vomit_or_chainsaw = matches!(injury_type_name, "ProjectileVomit" | "Chainsaw" | "ChainsawForSpp" | "FoulWithChainsaw" | "FoulForSppWithChainsaw");

        (no_attacker && vomit_or_chainsaw) || hurt_team_mate
    }

    fn has_prerequisite(game: &Game, ctx: &InjuryContext) -> bool {
        let player_id = if ctx.attacker_id.as_deref().map(|s| !s.is_empty()).unwrap_or(false) {
            ctx.attacker_id.as_deref()
        } else {
            ctx.defender_id.as_deref()
        };
        if let Some(id) = player_id {
            if let Some(player) = game.player(id) {
                return UtilServerReRoll::is_pro_re_roll_available(game, player);
            }
        }
        false
    }
}

impl Default for OldProModification {
    fn default() -> Self { Self::new() }
}

const VALID: &[&str] = &[
    "Block", "Chainsaw", "Foul", "FoulForSpp", "FoulForSppWithChainsaw",
    "FoulWithChainsaw", "BlockProne", "BlockStunned", "Stab", "ProjectileVomit",
];

impl InjuryContextModification for OldProModification {
    fn skill_use(&self) -> SkillUse { SkillUse::RE_ROLL_SINGLE_ARMOUR_DIE }
    fn valid_types(&self) -> &'static [&'static str] { VALID }
    fn requires_conditional_re_roll_skill(&self) -> bool { true }
    fn skill_id(&self) -> Option<u16> { self.skill_id }
    fn set_skill_id(&mut self, id: u16) { self.skill_id = Some(id); }

    fn allowed_for_attacker_and_defender_teams(&self, _game: &Game, _ctx: &InjuryContext) -> bool {
        true
    }

    /// OldPro overrides modify_armour entirely to use OldProModificationParams.
    /// We delegate the state into the base params and manage spotted_foul/self_inflicted inline.
    fn modify_armour(
        &self,
        game: &Game,
        rng: &mut GameRng,
        injury_ctx: &mut InjuryContext,
        injury_type_name: &'static str,
    ) -> bool {
        if injury_ctx.armor_roll.is_none() {
            return false;
        }
        let spotted_foul = Self::is_spotted_foul(injury_ctx, injury_type_name);
        let self_inflicted = Self::is_self_inflicted(game, injury_ctx, injury_type_name);

        if !Self::has_prerequisite(game, injury_ctx) {
            return false;
        }
        // Gate: armorBroken == selfInflicted OR spottedFoul
        let gate = (injury_ctx.armor_broken == self_inflicted) || spotted_foul;
        if !gate {
            return false;
        }

        let mut new_ctx = crate::injury::modification::injury_context_modification::clone_for_modification(injury_ctx, self.skill_use());

        // prepareArmourParams: detect spotted foul on new_ctx, pick replace index
        let spotted_foul2 = if let Some([d1, d2]) = new_ctx.armor_roll {
            d1 == d2 && matches!(injury_type_name, "Foul" | "FoulForSpp" | "FoulWithChainsaw" | "FoulForSppWithChainsaw")
        } else { false };

        let replace_index = if let Some([d1, d2]) = new_ctx.armor_roll {
            if (d1 < d2) != self_inflicted { 0 } else { 1 }
        } else { 0 };

        let old_value = new_ctx.armor_roll.map(|r| r[replace_index]).unwrap_or(0);

        // Set the forced value (1 if self-inflicted, 6 otherwise)
        if let Some(ref mut roll) = new_ctx.armor_roll {
            roll[replace_index] = if self_inflicted { 1 } else { 6 };
        }

        // Recalculate armor_broken
        if let Some(defender_id) = new_ctx.defender_id.clone() {
            let armor_value = game.player(&defender_id).map(|p| p.armour).unwrap_or(7);
            if let Some(roll) = new_ctx.armor_roll {
                use ffb_mechanics::mechanics::armor_broken;
                new_ctx.armor_broken = armor_broken(armor_value, roll, &new_ctx.armor_modifiers);
            }
        }

        // armourModificationCantHelp: !spottedFoul && (armorBroken == selfInflicted)
        if !spotted_foul2 && (new_ctx.armor_broken == self_inflicted) {
            return false;
        }

        // applyArmourModification: roll new die, insert, add report
        let new_value = rng.die(6);
        if let Some(ref mut roll) = new_ctx.armor_roll {
            roll[replace_index] = new_value;
        }

        // Recalculate armor_broken after rolling new die
        if let Some(defender_id) = new_ctx.defender_id.clone() {
            let armor_value = game.player(&defender_id).map(|p| p.armour).unwrap_or(7);
            if let Some(roll) = new_ctx.armor_roll {
                use ffb_mechanics::mechanics::armor_broken;
                new_ctx.armor_broken = armor_broken(armor_value, roll, &new_ctx.armor_modifiers);
            }
        }

        new_ctx.set_modification(ffb_model::injury::context::InjuryModification::ARMOUR);
        if let Some(id) = self.skill_id() {
            new_ctx.set_used_skill_id(id);
        }
        injury_ctx.modified_injury_context = Some(Box::new(new_ctx));
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{Rules, SkillId};
    use ffb_model::model::player::Player;
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use crate::step::framework::test_team;

    fn make() -> OldProModification { OldProModification::new() }

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    fn add_player_with_pro(game: &mut Game, home: bool, id: &str) {
        let mut p = Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "p".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 7,
            starting_skills: vec![SkillWithValue::new(SkillId::Pro)],
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0,
            race: None, ..Default::default()
        };
        if home { game.team_home.players.push(p); }
        else { game.team_away.players.push(p); }
    }

    fn ctx_with_roll(armor_roll: [i32; 2], armor_broken: bool) -> InjuryContext {
        let mut ctx = InjuryContext::new(ApothecaryMode::Defender);
        ctx.armor_roll = Some(armor_roll);
        ctx.armor_broken = armor_broken;
        ctx
    }

    #[test]
    fn requires_conditional_re_roll() {
        assert!(make().requires_conditional_re_roll_skill());
    }

    #[test]
    fn valid_types_include_block_foul_chainsaw() {
        let m = make();
        assert!(m.is_valid_type("Block"));
        assert!(m.is_valid_type("Foul"));
        assert!(m.is_valid_type("Chainsaw"));
    }

    #[test]
    fn modify_armour_returns_false_without_armor_roll() {
        let game = make_game();
        let mut rng = GameRng::new(1);
        let mut ctx = InjuryContext::new(ApothecaryMode::Defender);
        ctx.armor_roll = None;
        assert!(!make().modify_armour(&game, &mut rng, &mut ctx, "Block"));
    }

    #[test]
    fn is_spotted_foul_true_when_doubles_on_foul_type() {
        let ctx = ctx_with_roll([3, 3], false);
        assert!(OldProModification::is_spotted_foul(&ctx, "Foul"));
        assert!(OldProModification::is_spotted_foul(&ctx, "FoulForSpp"));
        assert!(OldProModification::is_spotted_foul(&ctx, "FoulWithChainsaw"));
    }

    #[test]
    fn is_spotted_foul_false_when_not_doubles() {
        let ctx = ctx_with_roll([3, 4], false);
        assert!(!OldProModification::is_spotted_foul(&ctx, "Foul"));
    }

    #[test]
    fn is_spotted_foul_false_for_non_foul_type() {
        let ctx = ctx_with_roll([4, 4], false);
        assert!(!OldProModification::is_spotted_foul(&ctx, "Block"));
        assert!(!OldProModification::is_spotted_foul(&ctx, "Chainsaw"));
    }

    #[test]
    fn is_self_inflicted_true_for_no_attacker_vomit() {
        let game = make_game();
        let mut ctx = ctx_with_roll([3, 4], false);
        ctx.attacker_id = None;
        assert!(OldProModification::is_self_inflicted(&game, &ctx, "ProjectileVomit"));
        assert!(OldProModification::is_self_inflicted(&game, &ctx, "Chainsaw"));
    }

    #[test]
    fn is_self_inflicted_false_for_no_attacker_non_vomit() {
        let game = make_game();
        let mut ctx = ctx_with_roll([3, 4], false);
        ctx.attacker_id = None;
        assert!(!OldProModification::is_self_inflicted(&game, &ctx, "Block"));
    }

    #[test]
    fn is_self_inflicted_true_for_same_team() {
        let mut game = make_game();
        add_player_with_pro(&mut game, true, "att");
        add_player_with_pro(&mut game, true, "def"); // both home → same team
        let mut ctx = ctx_with_roll([3, 4], false);
        ctx.attacker_id = Some("att".into());
        ctx.defender_id = Some("def".into());
        assert!(OldProModification::is_self_inflicted(&game, &ctx, "Block"));
    }

    #[test]
    fn has_prerequisite_false_when_no_pro_skill() {
        let mut game = make_game();
        // Player without Pro
        let p = Player {
            id: "p1".into(), name: "p1".into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 7,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0,
            race: None, ..Default::default()
        };
        game.team_home.players.push(p);
        let mut ctx = ctx_with_roll([3, 4], false);
        ctx.attacker_id = Some("p1".into());
        assert!(!OldProModification::has_prerequisite(&game, &ctx));
    }

    #[test]
    fn modify_armour_false_when_gate_not_met() {
        // gate = (armor_broken == self_inflicted) || spotted_foul
        // armor_broken=true, self_inflicted=false (no attacker, Block type) → gate = false
        // Wait: self_inflicted = false for block with attacker. We need armor_broken != self_inflicted.
        // armor_broken=true, self_inflicted=false → gate = (true == false) = false; spotted_foul=false → false
        let mut game = make_game();
        add_player_with_pro(&mut game, true, "att");
        add_player_with_pro(&mut game, false, "def"); // different teams
        game.turn_data_home.rerolls = 0; // no team reroll

        let mut rng = GameRng::new(1);
        let mut ctx = ctx_with_roll([5, 6], true); // armor_broken = true
        ctx.attacker_id = Some("att".into());
        ctx.defender_id = Some("def".into());
        // self_inflicted = false (different teams, not vomit)
        // gate = (true == false) = false; spotted_foul = false → gate = false
        // But also has_prerequisite must pass. Pro skill present but we also need game state.
        // is_pro_re_roll_available: player has CAN_REROLL_ONCE_PER_TURN + not used Pro yet.
        // Since player has Pro starting skill, it should return true IF the mechanic check passes.
        // Actually the check is pure: player.has_skill_property(CAN_REROLL_ONCE_PER_TURN) && !used_skills.contains(Pro)
        // So it should be true for our att player.
        assert!(!make().modify_armour(&game, &mut rng, &mut ctx, "Block"));
    }

    #[test]
    fn modify_armour_true_when_armor_broken_equals_self_inflicted_and_pro_available() {
        // armor_broken = false, self_inflicted = true (no attacker + ProjectileVomit)
        // gate = (false == true) = false || spotted_foul = false → gate = false? No wait:
        // We need gate = true. armor_broken == self_inflicted: false == true → false.
        // Let's use armor_broken=true, and force self_inflicted=true: same team
        // Then gate = (true == true) || false = true ✓
        let mut game = make_game();
        add_player_with_pro(&mut game, true, "att");
        add_player_with_pro(&mut game, true, "def"); // same team → self_inflicted = true

        let mut rng = GameRng::new(42);
        // armor_broken=true, self_inflicted=true → gate = true
        // armour_cant_help: !spotted_foul2 && (new_ctx.armor_broken == self_inflicted)
        // new_ctx forced roll: replace lower with 1 (self_inflicted), so [1, 6] for armour=7
        // armor_broken(7, [1, 6], []) → 1+6=7 not > 7 → false
        // So new_ctx.armor_broken == false, self_inflicted == true → false != true → can't help = true → return false?
        // Actually the modification "cant help" check is: !spotted_foul && (new_ctx.armor_broken == self_inflicted)
        // new_ctx.armor_broken = false (1+6=7, not > 7 with armour=7), self_inflicted = true
        // false == true → false → cant_help = false → continue to roll
        // Actually let me reconsider: we need armor value where [1, high_die] breaks armor.
        // With armour=4, [1,6]: 7 > 4 → breaks. new_ctx.armor_broken = true = self_inflicted → cant_help = true → false
        // With armour=4: new roll replaces lower(0 when self_inflicted: (d1<d2)!=self_inflicted = (t<f)=false → idx=1)
        // Wait: replace_index = if (d1 < d2) != self_inflicted { 0 } else { 1 }
        // d1=4, d2=5: d1<d2=true, self_inflicted=true → true != true = false → replace_index=1
        // forced value = 1 (self_inflicted). so [4, 1] → sum=5 > armour=4 → broken=true
        // new_ctx.armor_broken=true == self_inflicted=true → cant_help = true → return false
        // So we can't easily get a "returns true" case without knowing the exact rng seed.
        // Let's instead test the specific return value with known outcome.
        let mut ctx = ctx_with_roll([4, 5], true); // armor_broken=true
        ctx.attacker_id = Some("att".into());
        ctx.defender_id = Some("def".into());
        // We'll just check that with Pro available, modify_armour completes without panic
        // and sets modified_injury_context when the cant_help check passes.
        // With armour=7 (low), forced [4,1] → sum=5 ≤ 7 → not broken → new_ctx.armor_broken=false
        // cant_help: !false && (false == true) = true && false = false → proceed to roll
        // So with armour=7 defenders, gate passes and cant_help=false → should return true
        let result = make().modify_armour(&game, &mut rng, &mut ctx, "Block");
        // Either true (modification applied) or false (cant_help): depends on armour value (7)
        // Forced roll = 1 (self_inflicted), [4,1]→sum=5 ≤7 → not broken = false ≠ true(self_inflicted) → cant_help=false → returns true
        assert!(result, "With Pro and gate met and cant_help=false, should return true");
        assert!(ctx.modified_injury_context.is_some());
    }

    #[test]
    fn modify_armour_spotted_foul_gate_passes() {
        // spotted_foul = true (doubles on Foul) → gate = true regardless of other conditions
        let mut game = make_game();
        add_player_with_pro(&mut game, true, "att");
        add_player_with_pro(&mut game, false, "def"); // different teams: self_inflicted=false

        let mut rng = GameRng::new(99);
        let mut ctx = ctx_with_roll([3, 3], false); // doubles → spotted_foul=true; armor_broken=false
        ctx.attacker_id = Some("att".into());
        ctx.defender_id = Some("def".into());
        // gate = (armor_broken==self_inflicted) || spotted_foul = (false==false) || true = true
        // spotted_foul2: doubles on Foul → true
        // armour_cant_help: !spotted_foul2 && ... = !true && ... = false → never blocks
        // So should return true
        let result = make().modify_armour(&game, &mut rng, &mut ctx, "Foul");
        assert!(result);
        assert!(ctx.modified_injury_context.is_some());
    }
}
