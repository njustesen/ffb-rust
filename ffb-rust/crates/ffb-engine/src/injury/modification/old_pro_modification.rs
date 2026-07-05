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
    use ffb_model::enums::Rules;
    use crate::step::framework::test_team;

    fn make() -> OldProModification { OldProModification::new() }

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
        let game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025);
        let mut rng = GameRng::new(1);
        let mut ctx = InjuryContext::new(ApothecaryMode::Defender);
        ctx.armor_roll = None;
        assert!(!make().modify_armour(&game, &mut rng, &mut ctx, "Block"));
    }
}
