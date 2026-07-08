/// Translation of com.fumbbl.ffb.server.injury.modification.InjuryContextModification<T>.
///
/// Java: abstract generic class with default pipeline in modifyArmour() / modifyInjury().
/// Rust: trait with default method implementations; each concrete type overrides the hooks
/// it needs. The generic type parameter T (params) is replaced by ModificationParams
/// passed directly — OldPro stores its extra state as struct fields.
use ffb_model::enums::PlayerAction;
use ffb_model::model::SkillUse;
use ffb_model::injury::context::InjuryModification;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::injury::InjuryContext;
use crate::injury::modification::ModificationParams;
use crate::mechanic::roll_mechanic_for;

/// Java: IInjuryContextModification interface + InjuryContextModification abstract class.
pub trait InjuryContextModification: Send + Sync {
    // ── Identifying information ──────────────────────────────────────────────

    /// Java: skillUse() — which SkillUse enum value this modification represents.
    fn skill_use(&self) -> SkillUse;

    /// Java: validInjuryTypes — Set<Class<? extends InjuryType>>.
    /// Returns the Java simple class names this modification applies to.
    fn valid_types(&self) -> &'static [&'static str];

    /// Java: requiresConditionalReRollSkill() — default false; OldPro returns true.
    fn requires_conditional_re_roll_skill(&self) -> bool { false }

    /// Java: isValidType(InjuryType) — check if the injury type is in validInjuryTypes.
    fn is_valid_type(&self, injury_type_name: &str) -> bool {
        self.valid_types().contains(&injury_type_name)
    }

    // ── Skill identity ───────────────────────────────────────────────────────

    /// Java: getSkill() / setSkill() — the skill that owns this modification.
    /// Stored as numeric ID to avoid lifetime complexity.
    fn skill_id(&self) -> Option<u16>;
    fn set_skill_id(&mut self, id: u16);

    // ── Armour modification pipeline ─────────────────────────────────────────

    /// Java: modifyArmour(GameState, InjuryContext, InjuryType) → bool.
    ///
    /// Full pipeline: check ignoresArmourModifiersFromSkills, check
    /// allowedForAttackerAndDefenderTeams, call tryArmourRollModification,
    /// then modifyArmourInternal. On success: set modification + used_skill on the
    /// copy, store it as modified_injury_context.
    fn modify_armour(
        &self,
        game: &Game,
        rng: &mut GameRng,
        injury_ctx: &mut InjuryContext,
        injury_type_name: &'static str,
    ) -> bool {
        let defender_id = match injury_ctx.defender_id.as_deref() {
            Some(id) => id.to_owned(),
            None => return false,
        };
        // Java: UtilCards.hasUnusedSkillWithProperty(defender, ignoresArmourModifiersFromSkills)
        // Stub: always false (full implementation requires skill factory).
        if self.defender_ignores_armour_modifications(game, &defender_id) {
            return false;
        }

        let mut new_ctx = clone_for_modification(injury_ctx, self.skill_use());
        let allowed = self.allowed_for_attacker_and_defender_teams(game, injury_ctx);
        if !allowed {
            return false;
        }

        let mut params = ModificationParams::new(game, rng, new_ctx, injury_type_name);
        if self.try_armour_roll_modification(&params) && self.modify_armour_internal(&mut params) {
            params.new_context.set_modification(InjuryModification::ARMOUR);
            if let Some(id) = self.skill_id() {
                params.new_context.set_used_skill_id(id);
            }
            injury_ctx.modified_injury_context = Some(Box::new(params.new_context));
            return true;
        }
        false
    }

    /// Java: tryArmourRollModification(T params) — gate check; default false.
    fn try_armour_roll_modification(&self, params: &ModificationParams) -> bool { false }

    /// Java: modifyArmourInternal(T params) — applies prepareArmourParams, checks
    /// armourModificationCantHelp, then applyArmourModification.
    fn modify_armour_internal(&self, params: &mut ModificationParams) -> bool {
        self.prepare_armour_params(params);
        // Re-evaluate armor_broken after prepare step.
        let defender_id = params.new_context.defender_id.clone().unwrap_or_default();
        recalc_armor_broken_for_params(params, &defender_id);

        if self.armour_modification_cant_help(params) {
            return false;
        }

        params.new_context.clear_armor_modifiers();
        self.apply_armour_modification(params);
        recalc_armor_broken_for_params(params, &defender_id);
        true
    }

    /// Java: prepareArmourParams(T params) — default calls applyArmourModification.
    fn prepare_armour_params(&self, params: &mut ModificationParams) {
        self.apply_armour_modification(params);
    }

    /// Java: armourModificationCantHelp(T params) — default: can't help if armor NOT broken.
    fn armour_modification_cant_help(&self, params: &ModificationParams) -> bool {
        !params.new_context.armor_broken
    }

    /// Java: applyArmourModification(T params) — default: add skill's armor modifiers.
    fn apply_armour_modification(&self, params: &mut ModificationParams) {
        // Default: no-op (concrete types add their skill's modifiers in overrides).
        // Subclasses that use ADD_ARMOUR_MODIFIER override this.
    }

    // ── Injury modification pipeline ─────────────────────────────────────────

    /// Java: modifyInjury(GameState, InjuryContext, InjuryType) → bool.
    fn modify_injury(
        &self,
        game: &Game,
        rng: &mut GameRng,
        injury_ctx: &mut InjuryContext,
        injury_type_name: &'static str,
    ) -> bool {
        if !self.try_injury_modification(game, injury_ctx, injury_type_name) {
            return false;
        }
        let mut new_ctx = clone_for_modification(injury_ctx, self.skill_use());
        if self.allowed_for_attacker_and_defender_teams(game, injury_ctx)
            && self.modify_injury_internal(game, rng, &mut new_ctx)
        {
            new_ctx.set_modification(InjuryModification::INJURY);
            if let Some(id) = self.skill_id() {
                new_ctx.set_used_skill_id(id);
            }
            injury_ctx.modified_injury_context = Some(Box::new(new_ctx));
            return true;
        }
        false
    }

    /// Java: tryInjuryModification(Game, InjuryContext, InjuryType) — gate; default false.
    fn try_injury_modification(&self, game: &Game, ctx: &InjuryContext, injury_type_name: &str) -> bool { false }

    /// Java: modifyInjuryInternal(ModifiedInjuryContext, GameState) — default: add
    /// skill's injury modifiers and re-interpret the roll.
    fn modify_injury_internal(&self, game: &Game, _rng: &mut GameRng, ctx: &mut InjuryContext) -> bool {
        // Default: add injury modifiers from skill (ADD_INJURY_MODIFIER path).
        // Concrete types that RE_ROLL_INJURY override this entirely.
        let old_injury = ctx.injury;
        let new_injury = self.interpret_injury(game, ctx);
        if old_injury != new_injury {
            ctx.injury = new_injury;
            return true;
        }
        false
    }

    // ── Shared helpers ───────────────────────────────────────────────────────

    /// Java: allowedForAttackerAndDefenderTeams — default: differentTeams.
    fn allowed_for_attacker_and_defender_teams(&self, game: &Game, ctx: &InjuryContext) -> bool {
        self.different_teams(game, ctx)
    }

    /// Java: differentTeams — attacker is null OR attacker.team != defender.team.
    fn different_teams(&self, game: &Game, ctx: &InjuryContext) -> bool {
        let attacker_team = ctx.attacker_id.as_deref()
            .and_then(|id| game.player_team_id(id));
        let defender_team = ctx.defender_id.as_deref()
            .and_then(|id| game.player_team_id(id));
        attacker_team.is_none() || attacker_team != defender_team
    }

    /// Java: interpretInjury(GameState, InjuryContext) — re-run roll mechanic on context.
    fn interpret_injury(&self, game: &Game, ctx: &mut InjuryContext) -> Option<ffb_model::enums::PlayerState> {
        let mechanic = roll_mechanic_for(game.rules);
        mechanic.interpret_injury_roll(game, ctx)
    }

    /// Java: UtilCards.hasUnusedSkillWithProperty(defender, ignoresArmourModifiersFromSkills).
    /// Stub: always false until UtilCards is fully ported.
    fn defender_ignores_armour_modifications(&self, _game: &Game, _defender_id: &str) -> bool {
        false
    }

    /// Helper: is the acting player blitzing and in the attacker role?
    fn acting_player_is_blitzing_attacker(&self, game: &Game, ctx: &InjuryContext) -> bool {
        let acting_id = game.acting_player.player_id.as_deref();
        let attacker_id = ctx.attacker_id.as_deref();
        if acting_id.is_none() || acting_id != attacker_id {
            return false;
        }
        game.acting_player.player_action == Some(PlayerAction::Blitz)
    }

    /// Helper: acting player has tacklezones.
    fn acting_player_has_tacklezones(&self, game: &Game) -> bool {
        game.acting_player.player_id.as_deref()
            .and_then(|id| game.field_model.player_state(id))
            .map(|s| s.has_tacklezones())
            .unwrap_or(false)
    }
}

// ── Free helpers ─────────────────────────────────────────────────────────────

/// Clone the base InjuryContext into a new context for modification,
/// setting the skill_use field (Java: newContext.setSkillUse(skillUse())).
pub fn clone_for_modification(base: &InjuryContext, skill_use: SkillUse) -> InjuryContext {
    let mut ctx = base.clone();
    ctx.modified_injury_context = None; // ModifiedInjuryContext never nests further
    ctx.set_skill_use_modification(skill_use);
    ctx
}

/// Recalculate armor_broken for the new_context in a ModificationParams.
/// Java: DiceInterpreter.isArmourBroken(gameState, context) inside modifyArmourInternal.
fn recalc_armor_broken_for_params(params: &mut ModificationParams, defender_id: &str) {
    use ffb_mechanics::mechanics::armor_broken;
    if let Some(roll) = params.new_context.armor_roll {
        let armor_value = params.game.player(defender_id)
            .map(|p| p.armour)
            .unwrap_or(7);
        params.new_context.armor_broken = armor_broken(
            armor_value,
            roll,
            &params.new_context.armor_modifiers,
        );
    }
}
