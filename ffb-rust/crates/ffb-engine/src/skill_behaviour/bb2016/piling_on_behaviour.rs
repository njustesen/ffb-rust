/// 1:1 translation of com.fumbbl.ffb.server.skillbehaviour.bb2016.PilingOnBehaviour.
///
/// PilingOn is a BB2016-only skill (no BB2025 counterpart). The behaviour hooks into
/// `StepDropFallingPlayers` and operates in two phases:
///
/// **Phase 1 — initial pass (`state.usingPilingOn` is None):**
/// 1. Clear ROOTED flag on attacker/defender if FALLING.
/// 2. Drop the defender (call `UtilServerInjury.dropPlayer` for defender).
/// 3. Compute initial injury result for defender (InjuryTypeBlock / BlockStunned / BlockProne).
/// 4. If defender had `appliesPoisonOnBadlyHurt` and injury is badly hurt → roll WeepingDagger.
/// 5. Check all PilingOn conditions (attacker not FALLING, unused PilingOn skill, adjacent,
///    no immunity, options PILING_ON_INJURY_ONLY / PILING_ON_ARMOR_ONLY gate, etc.).
///    If all conditions pass → show `DialogPilingOn` and set `doNextStep = false`.
///
/// **Phase 2 — dialog responded (`state.usingPilingOn` is Some(bool)):**
/// 1. Publish `ReportPilingOn(playerId, usingPilingOn, reRollInjury)`.
/// 2. If using PilingOn (and team reroll not required, or team reroll available):
///    a. Mark skill used on attacker.
///    b. Drop attacker (`dropPlayer(attacker)`).
///    c. If `reRollInjury` → re-roll injury with `InjuryTypePilingOnInjury`, check for double
///       → possible KO attacker via `InjuryTypePilingOnKnockedOut` if `PILING_ON_TO_KO_ON_DOUBLE`.
///    d. Else → re-roll armour with `InjuryTypePilingOnArmour`, same double KO check.
///
/// **Attacker-falling path (same phase 1 / phase 2 gate):**
/// If the attacker is FALLING → publish END_TURN, drop attacker, compute attacker injury
/// (InjuryTypeBlock), check WeepingDagger for defender on badly hurt attacker.
///
/// **Step hooks:**
/// `handleCommandHook`: sets `state.usingPilingOn` from `useSkillCommand.isSkillUsed()`.
/// `handleExecuteStepHook`: full logic above; hooks into `StepDropFallingPlayers`.
///
/// TODO(hook-infra): `StepDropFallingPlayersHookState` not yet ported — this behaviour
///   stub is documented but the step modifier cannot yet execute.
use crate::skill_behaviour::SkillBehaviour;
use crate::model::skill_behaviour::SkillBehaviour as SbContainer;
use crate::skill_behaviour::registry::SkillRegistry;
use ffb_model::enums::SkillId;
use ffb_model::model::game::Game;

pub struct PilingOnBehaviour;

impl PilingOnBehaviour {
    pub fn new() -> Self { Self }

    /// Register the PilingOn skill into the skill registry.
    /// The step modifier is a stub until `StepDropFallingPlayersHookState` is available.
    pub fn register_into(registry: &mut SkillRegistry) {
        let sb = SbContainer::new();
        // No step modifier registered yet — full logic deferred to hook infra.
        registry.register(SkillId::PilingOn, sb);
    }
}

impl Default for PilingOnBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for PilingOnBehaviour {
    fn name(&self) -> &'static str { "PilingOnBehaviour" }

    fn execute_step_hook(&self, _game: &mut Game) -> bool {
        // TODO(hook-infra): implement once StepDropFallingPlayersHookState is ported
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use crate::step::framework::test_team;

    fn test_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2016)
    }

    #[test]
    fn name_returns_correct_string() {
        let b = PilingOnBehaviour::new();
        assert_eq!(b.name(), "PilingOnBehaviour");
    }

    #[test]
    fn default_has_correct_name() {
        let b = PilingOnBehaviour::default();
        assert_eq!(b.name(), "PilingOnBehaviour");
    }

    #[test]
    fn execute_step_hook_returns_false() {
        let b = PilingOnBehaviour::new();
        let mut game = test_game();
        assert!(!b.execute_step_hook(&mut game));
    }

    #[test]
    fn apply_modifier_is_noop() {
        use ffb_model::model::{Player, roster_position::RosterPosition};
        let b = PilingOnBehaviour::new();
        let mut player = Player::default();
        let pos = RosterPosition::default();
        let movement_before = player.movement;
        b.apply_modifier(&mut player, &pos);
        assert_eq!(player.movement, movement_before);
    }

    #[test]
    fn name_is_not_empty() {
        assert!(!PilingOnBehaviour::new().name().is_empty());
    }

    /// PilingOn is BB2016-only — no BB2025 equivalent skill exists.
    #[test]
    fn register_into_registers_piling_on_skill() {
        let mut reg = SkillRegistry::empty();
        PilingOnBehaviour::register_into(&mut reg);
        // Skill is registered (even though modifier is a stub)
        let sb = reg.get(SkillId::PilingOn).expect("PilingOn must be registered");
        // No step modifier yet — deferred until StepDropFallingPlayersHookState is ported
        assert_eq!(sb.get_step_modifiers().len(), 0,
            "PilingOn step modifier stub: no modifiers until hook infra is ported");
    }
}
