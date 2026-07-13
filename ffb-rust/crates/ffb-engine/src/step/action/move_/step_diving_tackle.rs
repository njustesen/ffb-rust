/// 1:1 translation of com.fumbbl.ffb.server.step.action.move.StepDivingTackle (COMMON rules)
/// plus its three per-edition hooks: com.fumbbl.ffb.server.skillbehaviour.{bb2016,bb2020,bb2025}
/// .DivingTackleBehaviour. Following the established "logic lives directly in the step" convention
/// (Dauntless/Wrestle/Stab/DumpOff precedent — see docs/PHASE_AAF_SKILL_HOOK_AUDIT.md), all three
/// editions' hook bodies are ported directly here rather than through the disconnected
/// `skill_behaviour/` registry mechanism.
///
/// Needs GOTO_LABEL_ON_SUCCESS init parameter.
/// Expects COORDINATE_FROM, COORDINATE_TO, DODGE_ROLL parameters.
///
/// BB2016 hook: 3-way branch (would-fail-regardless / fails-only-with-strength-modifier /
/// would-succeed-regardless), no stat-based-modifier axis.
///
/// BB2020/BB2025 hooks: identical bodies (near-verbatim in Java) apart from the eligible-tackler
/// lookup — BB2020 uses `findAdjacentOpposingPlayersWithProperty` + filterThrower/filterAttacker
/// AndDefender directly; BB2025 collapses this into `findEligibleDivingTacklers`, which adds a
/// destination-adjacency exclusion gated on `GameOptionId.DIVING_TACKLE_LEAVING_TZ_ONLY`. Both add
/// a second independent axis: a `StatBasedRollModifier` (in this codebase, only ever produced by
/// BB2020's "Incorporeal" — a Gretchen-only, once-per-game star-player skill; BB2025's
/// differently-scoped "Incorporeal" is an unrelated dodge-avoidance mechanic and never produces one).
///
/// client-only: the underlying Break-Tackle/Incorporeal dodge-modifier wiring in `step_move_dodge.rs`
/// is itself incomplete for BB2020/BB2025 (a separate, pre-existing gap — `DodgeModifierFactory::
/// find_skill_modifiers` only hardcodes bb2016's Break Tackle). DivingTackle reuses the same
/// modifier-computation primitives the base dodge step uses, so it inherits that limitation rather
/// than fixing it here (out of scope for this phase).
use ffb_model::enums::Rules;
use ffb_model::enums::SkillId;
use ffb_model::model::ActingPlayer;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::model::skill_use::SkillUse;
use ffb_model::option::game_option_id::DIVING_TACKLE_LEAVING_TZ_ONLY;
use ffb_model::option::util_game_option::is_option_enabled;
use ffb_model::report::report_skill_use::ReportSkillUse;
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_player::UtilPlayer;
use ffb_mechanics::modifiers::dodge_context::DodgeContext;
use ffb_mechanics::modifiers::dodge_modifier::DodgeModifier;
use ffb_mechanics::modifiers::dodge_modifier_factory::DodgeModifierFactory;
use ffb_mechanics::modifiers::modifier_type::ModifierType;
use ffb_mechanics::modifiers::player_stat_key::PlayerStatKey;
use ffb_mechanics::modifiers::stat_based_roll_modifier::StatBasedRollModifier;
use ffb_mechanics::modifiers::stat_based_roll_modifier_factory::StatBasedRollModifierFactory;
use crate::action::Action;
use crate::dice_interpreter::DiceInterpreter;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

pub struct StepDivingTackle {
    /// Java: state.goToLabelOnSuccess — GOTO_LABEL_ON_SUCCESS init parameter.
    pub goto_label_on_success: String,
    /// Java: state.coordinateFrom — set by COORDINATE_FROM parameter.
    pub coordinate_from: Option<FieldCoordinate>,
    /// Java: state.coordinateTo — set by COORDINATE_TO parameter.
    pub coordinate_to: Option<FieldCoordinate>,
    /// Java: state.dodgeRoll — set by DODGE_ROLL parameter.
    pub dodge_roll: i32,
    /// Java: state.usingDivingTackle — None = waiting for coach decision,
    /// Some(false) = not using, Some(true) = using.
    pub using_diving_tackle: Option<bool>,
    /// Java: state.usingBreakTackle
    pub using_break_tackle: bool,
    /// Java: state.usingModifyingSkill
    pub using_modifying_skill: Option<bool>,
}

impl StepDivingTackle {
    pub fn new() -> Self {
        Self {
            goto_label_on_success: String::new(),
            coordinate_from: None,
            coordinate_to: None,
            dodge_roll: 0,
            using_diving_tackle: None,
            using_break_tackle: false,
            using_modifying_skill: None,
        }
    }

    /// Java: `ActingPlayer.statBasedModifier(NamedProperties.canAddStrengthToDodge)`. Hardcoded to
    /// the one skill that currently registers this property: BB2020's `Incorporeal` (Gretchen-only,
    /// `ONCE_PER_GAME`, `setStatBasedRollModifierFactory(name, ST)`). BB2025's differently-scoped
    /// `Incorporeal` (registers `canAvoidDodging` instead — dodges are skipped entirely, an
    /// unrelated mechanic) never produces one, matching Java's edition split.
    fn stat_based_modifier_can_add_strength_to_dodge(
        game: &Game,
        acting: &ActingPlayer,
        rules: Rules,
    ) -> Option<StatBasedRollModifier> {
        if rules != Rules::Bb2020 {
            return None;
        }
        let player = acting.player_id.as_deref().and_then(|id| game.player(id))?;
        if player.has_skill(SkillId::Incorporeal) && !player.used_skills.contains(&SkillId::Incorporeal) {
            Some(StatBasedRollModifierFactory::new("Incorporeal", PlayerStatKey::ST).create(player))
        } else {
            None
        }
    }

    /// Computes the minimum dodge roll for a full modifier set: `DodgeModifierFactory.findModifiers`
    /// (`find_applicable` + `find_skill_modifiers`) plus, when `include_dt_modifier`, the "Diving
    /// Tackle" (+2, `DIVING_TACKLE` type) modifier — Java: `modifierFactory.forType(ModifierType
    /// .DIVING_TACKLE)`, hardcoded here since the only skill that ever registers a
    /// `DIVING_TACKLE`-type modifier is Diving Tackle itself (`skill/{bb2016,mixed}/
    /// DivingTackle.java`). Not every Java call site adds it — bb2016's post-success tail recheck
    /// omits it while every other call site includes it; see `finish`'s call site for the one place
    /// this varies.
    ///
    /// Returns `(minimum_roll, has_strength_modifier)`. `exclude_strength_modifier`, when true,
    /// removes the first use-strength modifier before computing (Java:
    /// `dodgeModifiers.remove(strengthModifier.get())`).
    ///
    /// Per-edition formula matches the real `AgilityMechanic.minimumRollDodge` bodies exactly
    /// (bb2016 swaps in strength for the base statistic when a use-strength modifier is present and
    /// ignores the stat-based-modifier argument entirely; bb2020/bb2025 always use agility and
    /// subtract the stat-based-modifier value). Implemented inline rather than via the
    /// `AgilityMechanic` trait because `DodgeModifier` has no `Hash`/`Eq` impl, so it cannot be
    /// placed in the `HashSet<DodgeModifier>` the trait's real signature requires — the same reason
    /// `step_move_dodge.rs` already bypasses the trait for its own minimum-roll computation.
    fn dodge_minimum_roll(
        game: &Game,
        rules: Rules,
        acting: &ActingPlayer,
        from: FieldCoordinate,
        to: FieldCoordinate,
        use_break_tackle: bool,
        stat_based_roll_modifier: Option<&StatBasedRollModifier>,
        exclude_strength_modifier: bool,
        include_dt_modifier: bool,
    ) -> (i32, bool) {
        let factory = DodgeModifierFactory::for_rules(rules);
        let ctx = DodgeContext::new_with_break_tackle(game, acting, from, to, use_break_tackle);
        let applicable = factory.find_applicable(&ctx);
        let skill_mods = factory.find_skill_modifiers(&ctx);
        let dt_modifier = DodgeModifier::new("Diving Tackle", 2, ModifierType::DIVING_TACKLE);
        let mut all: Vec<&DodgeModifier> = applicable.into_iter().chain(skill_mods.iter()).collect();
        if include_dt_modifier {
            all.push(&dt_modifier);
        }

        let has_strength = all.iter().any(|m| m.is_use_strength());
        if exclude_strength_modifier {
            if let Some(pos) = all.iter().position(|m| m.is_use_strength()) {
                all.remove(pos);
            }
        }

        let modifier_total: i32 = all.iter().map(|m| m.get_modifier()).sum();
        let stat_modifier = stat_based_roll_modifier.map(|s| s.get_modifier()).unwrap_or(0);
        let player = acting.player_id.as_deref().and_then(|id| game.player(id));

        let minimum_roll = match rules {
            Rules::Bb2016 => {
                let stat = if has_strength {
                    player.map(|p| p.strength_with_modifiers()).unwrap_or(0)
                } else {
                    player.map(|p| p.agility_with_modifiers()).unwrap_or(0)
                };
                (6 - stat.min(6) + modifier_total).max(2)
            }
            _ => {
                let agility = player.map(|p| p.agility_with_modifiers()).unwrap_or(0);
                (agility + modifier_total - stat_modifier).max(2)
            }
        };
        (minimum_roll, has_strength)
    }

    /// Java: `DivingTackleBehaviour(bb2016).handleExecuteStepHook`.
    fn execute_step_bb2016(&mut self, game: &mut Game) -> Option<StepOutcome> {
        let acting = game.acting_player.clone();
        if self.using_diving_tackle.is_none() {
            game.defender_id = None;
            self.using_diving_tackle = Some(false);
            if let Some(from) = self.coordinate_from {
                if game.field_model.player_at(from).is_none() {
                    let to = self.coordinate_to.unwrap_or(from);
                    // Java: findAdjacentOpposingPlayersWithProperty(..., checkAbleToMove = true)
                    let diving_tacklers: Vec<String> =
                        UtilPlayer::find_diving_tacklers(game, from, true).into_iter().cloned().collect();

                    if !diving_tacklers.is_empty() && self.dodge_roll > 0 {
                        let (minimum_roll, _) = Self::dodge_minimum_roll(
                            game, Rules::Bb2016, &acting, from, to, self.using_break_tackle, None, false, true,
                        );
                        let (minimum_roll_without_bt, has_strength) = Self::dodge_minimum_roll(
                            game, Rules::Bb2016, &acting, from, to, self.using_break_tackle, None, true, true,
                        );
                        let minimum_roll_without_bt = if has_strength { minimum_roll_without_bt } else { minimum_roll };

                        let mut descriptions = Vec::new();
                        let should_prompt;
                        if !DiceInterpreter::is_skill_roll_successful(self.dodge_roll, minimum_roll) {
                            should_prompt = true;
                        } else if !DiceInterpreter::is_skill_roll_successful(self.dodge_roll, minimum_roll_without_bt) {
                            descriptions.push(
                                "This will NOT trip the dodger, but will force the use of BREAK TACKLE.".to_string(),
                            );
                            should_prompt = true;
                        } else {
                            game.report_list.add(ReportSkillUse::new(
                                None, SkillId::DivingTackle, false, SkillUse::WOULD_NOT_HELP,
                            ));
                            should_prompt = false;
                        }

                        if should_prompt {
                            let team_id = game.inactive_team().id.clone();
                            self.using_diving_tackle = None;
                            return Some(StepOutcome::cont().with_prompt(
                                ffb_model::prompts::AgentPrompt::PlayerChoice {
                                    eligible_players: diving_tacklers,
                                    reason: team_id,
                                    descriptions,
                                },
                            ));
                        }
                    }
                }
            }
        }
        None
    }

    /// Java: `DivingTackleBehaviour(bb2020/bb2025).handleExecuteStepHook` (identical bodies apart
    /// from the eligible-tackler lookup).
    fn execute_step_stat_edition(&mut self, game: &mut Game, rules: Rules) -> Option<StepOutcome> {
        let acting = game.acting_player.clone();
        let used_stat_based_roll_modifier = if self.using_modifying_skill == Some(true) {
            Self::stat_based_modifier_can_add_strength_to_dodge(game, &acting, rules)
        } else {
            None
        };

        if self.using_diving_tackle.is_none() {
            game.defender_id = None;
            self.using_diving_tackle = Some(false);
            if let Some(from) = self.coordinate_from {
                if game.field_model.player_at(from).is_none() {
                    let to = self.coordinate_to.unwrap_or(from);
                    let diving_tacklers: Vec<String> = if rules == Rules::Bb2025 {
                        let leaving_tz_only = is_option_enabled(game, DIVING_TACKLE_LEAVING_TZ_ONLY);
                        UtilPlayer::find_eligible_diving_tacklers(game, from, to, leaving_tz_only)
                            .into_iter().cloned().collect()
                    } else {
                        UtilPlayer::find_diving_tacklers(game, from, false).into_iter().cloned().collect()
                    };

                    if !diving_tacklers.is_empty() && self.dodge_roll > 0 {
                        let (minimum_roll_with_current, strength_modifier_present) = Self::dodge_minimum_roll(
                            game, rules, &acting, from, to, self.using_break_tackle,
                            used_stat_based_roll_modifier.as_ref(), false, true,
                        );
                        let minimum_roll_without_bt = if strength_modifier_present {
                            Self::dodge_minimum_roll(
                                game, rules, &acting, from, to, self.using_break_tackle,
                                used_stat_based_roll_modifier.as_ref(), true, true,
                            ).0
                        } else {
                            minimum_roll_with_current
                        };

                        let mut descriptions: Vec<String> = Vec::new();
                        let should_prompt;

                        if !DiceInterpreter::is_skill_roll_successful(self.dodge_roll, minimum_roll_with_current) {
                            let available_stat_based_modifier =
                                Self::stat_based_modifier_can_add_strength_to_dodge(game, &acting, rules);

                            if strength_modifier_present {
                                if used_stat_based_roll_modifier.is_none() {
                                    if let Some(ref avail) = available_stat_based_modifier {
                                        let (required, _) = Self::dodge_minimum_roll(
                                            game, rules, &acting, from, to, self.using_break_tackle, Some(avail), false, true,
                                        );
                                        if DiceInterpreter::is_skill_roll_successful(self.dodge_roll, required) {
                                            descriptions.push(format!(
                                                "This will NOT trip the dodger, but will force the use of {}.",
                                                avail.get_report_string()
                                            ));
                                        }
                                    }
                                }
                            } else {
                                let (required, _dodge_modifiers_with_bt_has_strength) = Self::dodge_minimum_roll(
                                    game, rules, &acting, from, to, true,
                                    used_stat_based_roll_modifier.as_ref(), false, true,
                                );
                                // Java bug-for-bug: `strengthModifierCanBeAdded` re-checks the
                                // ORIGINAL (non-BT-forced) modifier set rather than the just-computed
                                // `dodgeModifiersWithBT` — in this branch (strengthModifier absent)
                                // that set never has a use-strength modifier, so it's always false.
                                // Faithfully reproduced rather than "fixed."
                                let strength_modifier_can_be_added = false;

                                if strength_modifier_can_be_added
                                    && DiceInterpreter::is_skill_roll_successful(self.dodge_roll, required)
                                {
                                    descriptions.push(
                                        "This will NOT trip the dodger, but will force the use of BREAK TACKLE.".to_string(),
                                    );
                                } else if used_stat_based_roll_modifier.is_none() {
                                    if let Some(ref avail) = available_stat_based_modifier {
                                        let (required2, _) = Self::dodge_minimum_roll(
                                            game, rules, &acting, from, to, true, Some(avail), false, true,
                                        );
                                        if DiceInterpreter::is_skill_roll_successful(self.dodge_roll, required2) {
                                            descriptions.push(if strength_modifier_can_be_added {
                                                format!(
                                                    "This will only trip the dodger, if BREAK TACKLE and {} are not used.",
                                                    avail.get_report_string()
                                                )
                                            } else {
                                                format!(
                                                    "This will only trip the dodger, if {} is not used.",
                                                    avail.get_report_string()
                                                )
                                            });
                                        }
                                    }
                                }
                            }
                            should_prompt = true;
                        } else if !DiceInterpreter::is_skill_roll_successful(self.dodge_roll, minimum_roll_without_bt) {
                            descriptions.push(
                                "This will NOT trip the dodger, but will force the use of BREAK TACKLE.".to_string(),
                            );
                            should_prompt = true;
                        } else {
                            game.report_list.add(ReportSkillUse::new(
                                None, SkillId::DivingTackle, false, SkillUse::WOULD_NOT_HELP,
                            ));
                            should_prompt = false;
                        }

                        if should_prompt {
                            let team_id = game.inactive_team().id.clone();
                            self.using_diving_tackle = None;
                            return Some(StepOutcome::cont().with_prompt(
                                ffb_model::prompts::AgentPrompt::PlayerChoice {
                                    eligible_players: diving_tacklers,
                                    reason: team_id,
                                    descriptions,
                                },
                            ));
                        }
                    }
                }
            }
        }
        None
    }

    /// Java: shared tail of all three `handleExecuteStepHook` bodies
    /// (`if (state.usingDivingTackle != null) { ... }`).
    fn finish(&mut self, game: &mut Game, rules: Rules) -> StepOutcome {
        let using = self.using_diving_tackle.unwrap_or(false);
        if self.using_diving_tackle.is_none() {
            return StepOutcome::next();
        }

        let outcome = StepOutcome::next().publish(StepParameter::UsingDivingTackle(using));
        if !using {
            return outcome;
        }

        // Java: recheck whether the dodge succeeds with Break Tackle triggered (ie. DT was used
        // only to force Break Tackle use) — if so, mark it as used.
        let acting = game.acting_player.clone();
        let from = self.coordinate_from.unwrap_or(FieldCoordinate::new(0, 0));
        let to = self.coordinate_to.unwrap_or(from);
        let used_stat_based_roll_modifier = if rules != Rules::Bb2016 && self.using_modifying_skill == Some(true) {
            Self::stat_based_modifier_can_add_strength_to_dodge(game, &acting, rules)
        } else {
            None
        };
        // Java: bb2016's tail recheck omits `.addAll(forType(DIVING_TACKLE))` (only the initial
        // decision block adds it there); bb2020/bb2025's tail recheck does add it. A genuine
        // asymmetry in the Java source, faithfully reproduced rather than unified.
        let (minimum_roll, has_strength) = Self::dodge_minimum_roll(
            game, rules, &acting, from, to, false, used_stat_based_roll_modifier.as_ref(), false,
            rules != Rules::Bb2016,
        );

        if has_strength && DiceInterpreter::is_skill_roll_successful(self.dodge_roll, minimum_roll) {
            self.using_break_tackle = true;
            if let Some(pid) = acting.player_id.as_deref() {
                if let Some(player) = game.player_mut(pid) {
                    player.used_skills.insert(SkillId::BreakTackle);
                }
            }
        }

        let defender_id = game.defender_id.clone();
        game.report_list.add(ReportSkillUse::new(
            defender_id, SkillId::DivingTackle, true, SkillUse::STOP_OPPONENT,
        ));

        let label = self.goto_label_on_success.clone();
        let mut result = StepOutcome::goto(&label).publish(StepParameter::UsingDivingTackle(true));
        if self.using_break_tackle {
            result = result.publish(StepParameter::UsingBreakTackle(true));
        }
        result
    }

    fn execute_step(&mut self, game: &mut Game) -> StepOutcome {
        let rules = game.rules;
        let prompt_outcome = match rules {
            Rules::Bb2016 => self.execute_step_bb2016(game),
            _ => self.execute_step_stat_edition(game, rules),
        };
        if let Some(outcome) = prompt_outcome {
            return outcome;
        }
        self.finish(game, rules)
    }
}

impl Default for StepDivingTackle {
    fn default() -> Self { Self::new() }
}

impl Step for StepDivingTackle {
    fn id(&self) -> StepId { StepId::DivingTackle }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: handleCommand receives CLIENT_PLAYER_CHOICE(DIVING_TACKLE) → usingDivingTackle.
        if let Action::PlayerChoice { player_id, mode, .. } = action {
            if mode == "DIVING_TACKLE" {
                self.using_diving_tackle = Some(player_id.is_some());
                game.defender_id = player_id.clone();
            }
        }
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnSuccess(v) => { self.goto_label_on_success = v.clone(); true }
            StepParameter::CoordinateFrom(v) => { self.coordinate_from = Some(*v); true }
            StepParameter::CoordinateTo(v) => { self.coordinate_to = Some(*v); true }
            StepParameter::DodgeRoll(v) => { self.dodge_roll = *v; true }
            StepParameter::UsingBreakTackle(v) => { self.using_break_tackle = *v; true }
            StepParameter::UsingModifyingSkill(v) => { self.using_modifying_skill = Some(*v); true }
            StepParameter::UsingDivingTackle(v) => { self.using_diving_tackle = Some(*v); true }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::StepAction;
    use ffb_model::model::game::Game;
    use ffb_model::model::player::Player;
    use ffb_model::enums::{PlayerType, PlayerGender, Rules};
    use ffb_model::model::SkillWithValue;
    use crate::step::framework::test_team;
    use ffb_model::util::rng::GameRng;

    fn make_game(rules: Rules) -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), rules)
    }

    fn add_player(
        game: &mut Game,
        home: bool,
        id: &str,
        coord: FieldCoordinate,
        agility: i32,
        strength: i32,
        skills: &[SkillId],
    ) {
        let player = Player {
            id: id.to_string(),
            name: id.to_string(),
            nr: 1,
            position_id: "pos".into(),
            player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            movement: 6,
            strength,
            agility,
            passing: 4,
            armour: 8,
            starting_skills: skills.iter().map(|&s| SkillWithValue::new(s)).collect(),
            extra_skills: vec![],
            temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0,
            stat_injuries: vec![],
            current_spps: 0,
            career_spps: 0,
            race: None,
            is_big_guy: false,
            ..Default::default()
        };
        let team = if home { &mut game.team_home } else { &mut game.team_away };
        team.players.push(player);
        game.field_model.set_player_coordinate(id, coord);
        // 0x100 = BIT_ACTIVE (private in player.rs) — "standing and able to act", matching the
        // `ACTIVE_STANDING` convention used by util_player.rs's own tests.
        game.field_model.set_player_state(id, ffb_model::enums::PlayerState(ffb_model::enums::PS_STANDING | 0x100));
    }

    fn seed_for_d6(target: i32, max_tries: u64) -> u64 {
        for seed in 0..max_tries {
            let mut rng = GameRng::new(seed);
            if rng.d6() == target {
                return seed;
            }
        }
        panic!("no seed found for target {target}");
    }

    #[test]
    fn id_is_diving_tackle() {
        assert_eq!(StepDivingTackle::new().id(), StepId::DivingTackle);
    }

    #[test]
    fn parameters_stored_correctly() {
        let mut step = StepDivingTackle::new();
        let coord = FieldCoordinate::new(3, 5);
        assert!(step.set_parameter(&StepParameter::GotoLabelOnSuccess("S".into())));
        assert!(step.set_parameter(&StepParameter::CoordinateFrom(coord)));
        assert!(step.set_parameter(&StepParameter::CoordinateTo(coord)));
        assert!(step.set_parameter(&StepParameter::DodgeRoll(3)));
        assert!(step.set_parameter(&StepParameter::UsingBreakTackle(true)));
        assert!(step.set_parameter(&StepParameter::UsingModifyingSkill(false)));
        assert!(step.set_parameter(&StepParameter::UsingDivingTackle(true)));
        assert_eq!(step.goto_label_on_success, "S");
        assert_eq!(step.dodge_roll, 3);
        assert!(step.using_break_tackle);
        assert_eq!(step.using_diving_tackle, Some(true));
    }

    #[test]
    fn unrecognised_parameter_returns_false() {
        assert!(!StepDivingTackle::new().set_parameter(&StepParameter::EndTurn(true)));
    }

    #[test]
    fn no_coordinate_from_returns_next_step() {
        let mut game = make_game(Rules::Bb2025);
        let mut step = StepDivingTackle::new();
        step.goto_label_on_success = "DT".into();
        let outcome = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
        assert_eq!(step.using_diving_tackle, Some(false));
    }

    #[test]
    fn no_eligible_tacklers_returns_next_step() {
        let mut game = make_game(Rules::Bb2025);
        let mut step = StepDivingTackle::new();
        step.goto_label_on_success = "DT".into();
        step.coordinate_from = Some(FieldCoordinate::new(5, 5));
        step.coordinate_to = Some(FieldCoordinate::new(6, 5));
        step.dodge_roll = 4;
        let outcome = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
        assert_eq!(step.using_diving_tackle, Some(false));
    }

    #[test]
    fn preset_using_diving_tackle_true_goes_to_label() {
        let mut game = make_game(Rules::Bb2025);
        game.acting_player.player_id = Some("home1".into());
        add_player(&mut game, true, "home1", FieldCoordinate::new(5, 5), 3, 3, &[]);
        let mut step = StepDivingTackle::new();
        step.goto_label_on_success = "DT_LABEL".into();
        step.using_diving_tackle = Some(true);
        step.dodge_roll = 2;
        let outcome = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::GotoLabel);
        assert_eq!(outcome.goto_label.as_deref(), Some("DT_LABEL"));
        assert!(game.report_list.has_report(ffb_model::report::report_id::ReportId::SKILL_USE));
    }

    #[test]
    fn preset_using_diving_tackle_false_returns_next_step() {
        let mut game = make_game(Rules::Bb2025);
        let mut step = StepDivingTackle::new();
        step.using_diving_tackle = Some(false);
        let outcome = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
        assert!(outcome.published.iter().any(|p| matches!(p, StepParameter::UsingDivingTackle(false))));
    }

    #[test]
    fn bb2016_eligible_tackler_prompts_when_roll_would_fail() {
        let mut game = make_game(Rules::Bb2016);
        game.acting_player.player_id = Some("home1".into());
        // home1 has already vacated coordinate_from (5,5) — Java requires the source square empty.
        add_player(&mut game, true, "home1", FieldCoordinate::new(6, 6), 3, 3, &[]);
        add_player(&mut game, false, "away1", FieldCoordinate::new(6, 5), 3, 3, &[SkillId::DivingTackle]);
        let mut step = StepDivingTackle::new();
        step.goto_label_on_success = "DT".into();
        step.coordinate_from = Some(FieldCoordinate::new(5, 5));
        step.coordinate_to = Some(FieldCoordinate::new(6, 6));
        // agility 3, base = 6-3=3, +2 (DivingTackle) = 5 minimum → dodge_roll=3 fails.
        step.dodge_roll = 3;
        let outcome = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::Continue);
        assert!(outcome.prompt.is_some());
        assert_eq!(step.using_diving_tackle, None);
    }

    #[test]
    fn bb2016_would_not_help_reports_and_continues() {
        let mut game = make_game(Rules::Bb2016);
        game.acting_player.player_id = Some("home1".into());
        add_player(&mut game, true, "home1", FieldCoordinate::new(6, 6), 6, 3, &[]);
        add_player(&mut game, false, "away1", FieldCoordinate::new(6, 5), 3, 3, &[SkillId::DivingTackle]);
        let mut step = StepDivingTackle::new();
        step.goto_label_on_success = "DT".into();
        step.coordinate_from = Some(FieldCoordinate::new(5, 5));
        step.coordinate_to = Some(FieldCoordinate::new(6, 6));
        // agility 6 → base = 6-6=0, +2 DT = 2 → any roll succeeds → WOULD_NOT_HELP, no prompt.
        step.dodge_roll = 6;
        let outcome = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
        assert_eq!(step.using_diving_tackle, Some(false));
        assert!(game.report_list.has_report(ffb_model::report::report_id::ReportId::SKILL_USE));
    }

    #[test]
    fn handle_command_accepts_diving_tackle_choice() {
        let mut game = make_game(Rules::Bb2016);
        game.acting_player.player_id = Some("home1".into());
        add_player(&mut game, true, "home1", FieldCoordinate::new(5, 5), 3, 3, &[]);
        add_player(&mut game, false, "away1", FieldCoordinate::new(6, 5), 3, 3, &[SkillId::DivingTackle]);
        let mut step = StepDivingTackle::new();
        step.goto_label_on_success = "DT_LABEL".into();
        step.coordinate_from = Some(FieldCoordinate::new(5, 5));
        step.coordinate_to = Some(FieldCoordinate::new(6, 6));
        step.using_diving_tackle = None;
        let outcome = step.handle_command(
            &Action::PlayerChoice { player_id: Some("away1".into()), player_ids: vec![], mode: "DIVING_TACKLE".into() },
            &mut game,
            &mut GameRng::new(0),
        );
        assert_eq!(game.defender_id.as_deref(), Some("away1"));
        assert_eq!(outcome.action, StepAction::GotoLabel);
    }

    #[test]
    fn handle_command_declines_diving_tackle_choice() {
        let mut game = make_game(Rules::Bb2016);
        let mut step = StepDivingTackle::new();
        step.using_diving_tackle = None;
        let outcome = step.handle_command(
            &Action::PlayerChoice { player_id: None, player_ids: vec![], mode: "DIVING_TACKLE".into() },
            &mut game,
            &mut GameRng::new(0),
        );
        assert_eq!(outcome.action, StepAction::NextStep);
        assert_eq!(step.using_diving_tackle, Some(false));
    }

    fn bb2025_leaving_tz_only_scenario(leaving_tz_only: bool) -> StepOutcome {
        let mut game = make_game(Rules::Bb2025);
        game.acting_player.player_id = Some("home1".into());
        if leaving_tz_only {
            game.options.set(DIVING_TACKLE_LEAVING_TZ_ONLY, "true");
        }
        add_player(&mut game, true, "home1", FieldCoordinate::new(10, 10), 3, 3, &[]);
        // away1 is adjacent to both from(5,5) and to(5,6) — excluded only when leaving_tz_only.
        add_player(&mut game, false, "away1", FieldCoordinate::new(6, 6), 3, 3, &[SkillId::DivingTackle]);
        let mut step = StepDivingTackle::new();
        step.goto_label_on_success = "DT".into();
        step.coordinate_from = Some(FieldCoordinate::new(5, 5));
        step.coordinate_to = Some(FieldCoordinate::new(5, 6));
        step.dodge_roll = 1;
        step.start(&mut game, &mut GameRng::new(0))
    }

    #[test]
    fn bb2025_without_leaving_tz_only_prompts_for_adjacent_tackler() {
        let outcome = bb2025_leaving_tz_only_scenario(false);
        assert_eq!(outcome.action, StepAction::Continue);
        assert!(outcome.prompt.is_some());
    }

    #[test]
    fn bb2025_leaving_tz_only_excludes_destination_adjacent_tackler() {
        let outcome = bb2025_leaving_tz_only_scenario(true);
        assert_eq!(outcome.action, StepAction::NextStep);
    }

    #[test]
    fn bb2020_eligible_tackler_prompts_when_roll_would_fail() {
        let mut game = make_game(Rules::Bb2020);
        game.acting_player.player_id = Some("home1".into());
        add_player(&mut game, true, "home1", FieldCoordinate::new(6, 6), 3, 3, &[]);
        add_player(&mut game, false, "away1", FieldCoordinate::new(6, 5), 3, 3, &[SkillId::DivingTackle]);
        let mut step = StepDivingTackle::new();
        step.goto_label_on_success = "DT".into();
        step.coordinate_from = Some(FieldCoordinate::new(5, 5));
        step.coordinate_to = Some(FieldCoordinate::new(6, 6));
        step.dodge_roll = 1;
        let outcome = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::Continue);
        assert!(outcome.prompt.is_some());
        assert_eq!(step.using_diving_tackle, None);
    }

    #[test]
    fn bb2016_break_tackle_marked_used_when_it_alone_secures_dodge() {
        let mut game = make_game(Rules::Bb2016);
        game.acting_player.player_id = Some("home1".into());
        add_player(&mut game, true, "home1", FieldCoordinate::new(6, 6), 3, 5, &[SkillId::BreakTackle]);
        let mut step = StepDivingTackle::new();
        step.goto_label_on_success = "DT_LABEL".into();
        step.coordinate_from = Some(FieldCoordinate::new(5, 5));
        step.coordinate_to = Some(FieldCoordinate::new(6, 6));
        step.using_diving_tackle = Some(true);
        // bb2016 Break Tackle modifier value is 0 (hardcoded use-strength swap only) — with it,
        // minimum roll uses strength (5) instead of agility (3): 6 - min(5,6) = 1, floored at 2.
        step.dodge_roll = 2;
        let outcome = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::GotoLabel);
        assert!(step.using_break_tackle);
        assert!(outcome.published.iter().any(|p| matches!(p, StepParameter::UsingBreakTackle(true))));
        assert!(game.player("home1").unwrap().used_skills.contains(&SkillId::BreakTackle));
    }

    #[test]
    fn seed_helper_finds_target() {
        let seed = seed_for_d6(4, 10_000);
        let mut rng = GameRng::new(seed);
        assert_eq!(rng.d6(), 4);
    }
}
