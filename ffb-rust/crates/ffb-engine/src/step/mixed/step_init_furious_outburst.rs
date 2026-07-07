/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.StepInitFuriousOutburst`.
///
/// Initialises the Furious Outburst sequence (star player "Then I Started Blastin'").
/// Needs `GOTO_LABEL_ON_END` init parameter.
///
/// Java logic (executeStep):
///   1. Get acting player + unused `canTeleportBeforeAndAfterAvRollAttack` skill.
///   2. If not prone/stunned:
///      a. If `end_turn`: publish END_TURN + CHECK_FORGO → goto end label.
///      b. If `end_player_action`: publish END_PLAYER_ACTION + cancel target selection → goto.
///      c. If skill != null:
///         - If `target_id` is set → set TargetSelectionState + next_step.
///         - Otherwise → find eligible players (blockable within 3, with empty adj. sq.),
///           show dialog (FURIOUS_OUTBURST) → CONTINUE.
///   3. Otherwise (prone/stunned) → goto end label.
///
/// TargetSelectionState, UtilPlayer, and dialog are not yet ported — stubbed.
///
/// Java: `StepInitFuriousOutburst extends AbstractStep` (mixed, BB2020 + BB2025).
use std::collections::HashSet;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::model::skill_def::SkillWithValue;
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_cards::UtilCards;
use ffb_model::util::util_player::UtilPlayer;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepInitFuriousOutburst` (mixed, BB2020 + BB2025).
pub struct StepInitFuriousOutburst {
    /// Java: `eligiblePlayers` — player IDs selectable in the dialog.
    pub eligible_players: HashSet<String>,
    /// Java: `endPlayerAction`
    pub end_player_action: bool,
    /// Java: `endTurn`
    pub end_turn: bool,
    /// Java: `goToLabelOnEnd` (mandatory init param GOTO_LABEL_ON_END)
    pub goto_label_on_end: String,
    /// Java: `targetId` — chosen target player ID.
    pub target_id: Option<String>,
}

impl StepInitFuriousOutburst {
    pub fn new(goto_label_on_end: impl Into<String>) -> Self {
        Self {
            eligible_players: HashSet::new(),
            end_player_action: false,
            end_turn: false,
            goto_label_on_end: goto_label_on_end.into(),
            target_id: None,
        }
    }

    fn execute_step(&mut self, game: &mut Game) -> StepOutcome {
        let acting_id = game.acting_player.player_id.clone();
        let acting_id = match acting_id {
            Some(id) => id,
            None => return StepOutcome::goto(&self.goto_label_on_end),
        };

        let player_state = game.field_model.player_state(&acting_id);
        let prone_or_stunned = player_state.map(|s| s.is_prone_or_stunned()).unwrap_or(false);

        if !prone_or_stunned {
            // Java: Skill skill = UtilCards.getUnusedSkillWithProperty(actingPlayer, canTeleportBeforeAndAfterAvRollAttack)
            let skill = game.acting_player.player_id.as_deref()
                .and_then(|id| game.player(id))
                .and_then(|p| UtilCards::get_unused_skill_with_property(p, NamedProperties::CAN_TELEPORT_BEFORE_AND_AFTER_AV_ROLL_ATTACK));

            if self.end_turn {
                return StepOutcome::goto(&self.goto_label_on_end)
                    .publish(StepParameter::EndTurn(true))
                    .publish(StepParameter::CheckForgo(true));
            }
            if self.end_player_action {
                // Java: fieldModel.setTargetSelectionState(new TargetSelectionState().cancel())
                let mut ts = ffb_model::model::target_selection_state::TargetSelectionState::default();
                ts.cancel();
                game.field_model.target_selection_state = Some(ts);
                return StepOutcome::goto(&self.goto_label_on_end)
                    .publish(StepParameter::EndPlayerAction(true));
            }
            if skill.is_some() {
                if let Some(ref tid) = self.target_id.clone() {
                    // Java: game.fieldModel.setTargetSelectionState(new TargetSelectionState(targetId))
                    let ts = ffb_model::model::target_selection_state::TargetSelectionState::new(tid.clone());
                    game.field_model.target_selection_state = Some(ts);
                    return StepOutcome::next();
                }
                // Java: findEligiblePlayers() — opponents within 3 with an empty adjacent square
                let found = Self::find_eligible_players(game, &acting_id);
                if !found.is_empty() {
                    self.eligible_players.extend(found);
                    // client-only: DialogPlayerChoiceParameter(FURIOUS_OUTBURST) — headless falls through
                    return StepOutcome::cont();
                }
            }
        }

        StepOutcome::goto(&self.goto_label_on_end)
    }

    /// Java: findEligiblePlayers — opponents within 3 squares of acting player that have an empty adjacent square.
    fn find_eligible_players(game: &Game, acting_id: &str) -> Vec<String> {
        let Some(coord) = game.field_model.player_coordinate(acting_id) else {
            return Vec::new();
        };
        let opponent_team = game.inactive_team();
        UtilPlayer::find_blockable_players(game, opponent_team, coord, 3)
            .into_iter()
            .filter(|pid| Self::has_empty_adjacent_square(game, pid))
            .map(|pid| pid.clone())
            .collect()
    }

    fn has_empty_adjacent_square(game: &Game, player_id: &str) -> bool {
        let Some(coord) = game.field_model.player_coordinate(player_id) else {
            return false;
        };
        game.field_model.adjacent_on_pitch(coord)
            .into_iter()
            .any(|c| game.field_model.player_at(c).is_none())
    }
}

impl Default for StepInitFuriousOutburst {
    fn default() -> Self { Self::new("") }
}

impl Step for StepInitFuriousOutburst {
    fn id(&self) -> StepId { StepId::InitFuriousOutburst }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        match action {
            // Java: CLIENT_PLAYER_CHOICE
            Action::PlayerChoice { player_id, .. } => {
                match player_id {
                    None => { self.end_player_action = true; }
                    Some(pid) if self.eligible_players.contains(pid.as_str()) => {
                        self.target_id = Some(pid.clone());
                    }
                    _ => {}
                }
            }
            Action::EndTurn => { self.end_turn = true; }
            _ => {}
        }
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::EndPlayerAction(v) => { self.end_player_action = *v; true }
            StepParameter::GotoLabelOnEnd(v) => { self.goto_label_on_end = v.clone(); true }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{test_team, StepAction};
    use ffb_model::enums::{Rules, PS_STANDING, PS_PRONE, PlayerAction, SkillId};
    use ffb_model::model::game::Game;
    use ffb_model::model::player::Player;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use ffb_model::types::FieldCoordinate;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    fn add_player(game: &mut Game, id: &str, state: u32) {
        add_player_with_skills(game, id, state, vec![]);
    }

    fn add_player_with_skills(game: &mut Game, id: &str, state: u32, skills: Vec<SkillId>) {
        let pos = FieldCoordinate::new(5, 5);
        game.team_home.players.push(Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: skills.into_iter().map(|s| SkillWithValue { skill_id: s, value: None }).collect(),
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        });
        game.field_model.set_player_coordinate(id, pos);
        game.field_model.set_player_state(id, ffb_model::enums::PlayerState::new(state));
        game.acting_player.set_player(id.into(), PlayerAction::Block);
    }

    #[test]
    fn id_is_init_furious_outburst() {
        assert_eq!(StepInitFuriousOutburst::new("end").id(), StepId::InitFuriousOutburst);
    }

    #[test]
    fn prone_player_goes_to_label() {
        let mut step = StepInitFuriousOutburst::new("end_label");
        let mut game = make_game();
        add_player(&mut game, "att", PS_PRONE);
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label, Some("end_label".into()));
    }

    #[test]
    fn end_turn_publishes_end_turn_and_check_forgo() {
        let mut step = StepInitFuriousOutburst::new("end");
        step.end_turn = true;
        let mut game = make_game();
        add_player(&mut game, "att", PS_STANDING);
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::GotoLabel);
        let has_end_turn = out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true)));
        let has_check_forgo = out.published.iter().any(|p| matches!(p, StepParameter::CheckForgo(true)));
        assert!(has_end_turn && has_check_forgo);
    }

    #[test]
    fn end_player_action_publishes_epa() {
        let mut step = StepInitFuriousOutburst::new("end");
        step.end_player_action = true;
        let mut game = make_game();
        add_player(&mut game, "att", PS_STANDING);
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::GotoLabel);
        let has_epa = out.published.iter().any(|p| matches!(p, StepParameter::EndPlayerAction(true)));
        assert!(has_epa);
    }

    #[test]
    fn end_player_action_cancels_target_selection_state() {
        let mut step = StepInitFuriousOutburst::new("end");
        step.end_player_action = true;
        let mut game = make_game();
        add_player(&mut game, "att", PS_STANDING);
        let mut rng = GameRng::new(0);
        step.start(&mut game, &mut rng);
        assert!(game.field_model.target_selection_state.as_ref().map(|ts| ts.is_canceled()).unwrap_or(false));
    }

    #[test]
    fn target_id_sets_target_selection_state() {
        let mut step = StepInitFuriousOutburst::new("end");
        step.target_id = Some("tgt".into());
        let mut game = make_game();
        // Player must have FuriousOutburst skill to enter the skill branch
        add_player_with_skills(&mut game, "att", PS_STANDING, vec![SkillId::FuriousOutburst]);
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
        let player_id = game.field_model.target_selection_state.as_ref()
            .and_then(|ts| ts.get_selected_player_id().cloned());
        assert_eq!(player_id.as_deref(), Some("tgt"));
    }

    #[test]
    fn no_furious_outburst_skill_goes_to_label() {
        // Without the FuriousOutburst skill, the step skips to goto even if target_id is set
        let mut step = StepInitFuriousOutburst::new("end_label");
        step.target_id = Some("tgt".into());
        let mut game = make_game();
        add_player(&mut game, "att", PS_STANDING); // no FuriousOutburst skill
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label, Some("end_label".into()));
    }
}
