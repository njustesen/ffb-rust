/// BB2025 Throw Team-Mate step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2025.ThrowTeamMate`.
use ffb_model::enums::ApothecaryMode;
use ffb_model::types::FieldCoordinate;
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};
use super::activation_sequence_builder::ActivationSequenceBuilder;

#[derive(Debug, Clone, Default)]
pub struct ThrowTeamMateParams {
    pub thrown_player_id: Option<String>,
    pub is_kicked: bool,
    pub target_coordinate: Option<FieldCoordinate>,
}

pub struct ThrowTeamMate;

impl ThrowTeamMate {
    pub fn new() -> Self { Self }

    pub fn build_sequence(params: &ThrowTeamMateParams) -> Vec<SequenceStep> {
        let mut seq = Sequence::new();
        // 1 INIT_THROW_TEAM_MATE
        let mut init_p = vec![
            StepParameter::GotoLabelOnEnd(labels::END_THROW_TEAM_MATE.into()),
            StepParameter::IsKickedPlayer(params.is_kicked),
        ];
        if let Some(ref id) = params.thrown_player_id {
            init_p.push(StepParameter::ThrownPlayerId(Some(id.clone())));
        }
        if let Some(coord) = params.target_coordinate {
            init_p.push(StepParameter::TargetCoordinate(coord));
        }
        seq.add(StepId::InitThrowTeamMate, init_p);

        // 2-14 [ACTIVATION(END_THROW_TEAM_MATE; eventual defender = thrown player)]
        ActivationSequenceBuilder::new()
            .with_failure_label(labels::END_THROW_TEAM_MATE)
            .with_eventual_defender(params.thrown_player_id.clone())
            .add_to(&mut seq);

        // ALWAYS_HUNGRY (failure → EAT_TEAM_MATE, success → RESOLVE_PASS)
        seq.add(StepId::AlwaysHungry, vec![
            StepParameter::IsKickedPlayer(params.is_kicked),
            StepParameter::GotoLabelOnFailure(labels::EAT_TEAM_MATE.into()),
            StepParameter::GotoLabelOnSuccess(labels::RESOLVE_PASS.into()),
        ]);
        // THROW_TEAM_MATE
        seq.add(StepId::ThrowTeamMate, vec![
            StepParameter::IsKickedPlayer(params.is_kicked),
        ]);
        // DISPATCH_SCATTER_PLAYER [RESOLVE_PASS]
        seq.add_labelled(StepId::DispatchScatterPlayer, labels::RESOLVE_PASS, vec![
            StepParameter::IsKickedPlayer(params.is_kicked),
        ]);
        // RIGHT_STUFF [RIGHT_STUFF]
        seq.add_labelled(StepId::RightStuff, labels::RIGHT_STUFF, vec![
            StepParameter::IsKickedPlayer(params.is_kicked),
            StepParameter::GotoLabelOnSuccess(labels::END_SCATTER_PLAYER.into()),
        ]);
        // STEADY_FOOTING (goto END_SCATTER_PLAYER on success)
        seq.add(StepId::SteadyFooting, vec![
            StepParameter::GotoLabelOnSuccess(labels::END_SCATTER_PLAYER.into()),
        ]);
        // GOTO_LABEL → APOTHECARY_THROWN_PLAYER
        seq.jump(labels::APOTHECARY_THROWN_PLAYER);
        // EAT_TEAM_MATE [EAT_TEAM_MATE]
        seq.add_labelled(StepId::EatTeamMate, labels::EAT_TEAM_MATE, vec![]);
        // APOTHECARY [APOTHECARY_THROWN_PLAYER] (THROWN_PLAYER)
        seq.add_labelled(StepId::Apothecary, labels::APOTHECARY_THROWN_PLAYER, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::ThrownPlayer),
        ]);
        // CATCH_SCATTER_THROW_IN [END_SCATTER_PLAYER]
        seq.add_labelled(StepId::CatchScatterThrowIn, labels::END_SCATTER_PLAYER, vec![]);
        // RESET_TO_MOVE [END_THROW_TEAM_MATE]
        seq.add_labelled(StepId::ResetToMove, labels::END_THROW_TEAM_MATE, vec![]);
        // END_THROW_TEAM_MATE
        seq.add(StepId::EndThrowTeamMate, vec![]);
        seq.build()
    }
}

impl Default for ThrowTeamMate {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn throw_team_mate_has_25_steps_with_activation() {
        // Java pushSequence: INIT_THROW_TEAM_MATE (1) + ActivationSequenceBuilder w/o eventual
        // defender (13, no SET_DEFENDER since thrown_player_id is None) + 11 own steps = 25.
        let steps = ThrowTeamMate::build_sequence(&ThrowTeamMateParams::default());
        assert_eq!(steps.len(), 25);
        assert_eq!(steps[1].step_id, StepId::InitActivation);
    }

    #[test]
    fn throw_team_mate_sets_eventual_defender_when_thrown_player_id_present() {
        // Java: ActivationSequenceBuilder.create()...withEventualDefender(params.getThrownPlayerId())
        // adds a SET_DEFENDER step when the thrown player id is known.
        let params = ThrowTeamMateParams { thrown_player_id: Some("p9".into()), ..Default::default() };
        let steps = ThrowTeamMate::build_sequence(&params);
        assert_eq!(steps.len(), 26);
        let sd = steps.iter().find(|s| s.step_id == StepId::SetDefender).unwrap();
        assert!(sd.params.iter().any(|p| matches!(p, StepParameter::BlockDefenderId(id) if id == "p9")));
    }

    #[test]
    fn throw_team_mate_ends_with_end_throw_team_mate() {
        let steps = ThrowTeamMate::build_sequence(&ThrowTeamMateParams::default());
        assert_eq!(steps.last().unwrap().step_id, StepId::EndThrowTeamMate);
    }

    #[test]
    fn throw_team_mate_eat_team_mate_is_labelled() {
        let steps = ThrowTeamMate::build_sequence(&ThrowTeamMateParams::default());
        let etm = steps.iter().find(|s| s.step_id == StepId::EatTeamMate).unwrap();
        assert_eq!(etm.label.as_deref(), Some(labels::EAT_TEAM_MATE));
    }

    #[test]
    fn throw_team_mate_apothecary_thrown_player_is_labelled() {
        let steps = ThrowTeamMate::build_sequence(&ThrowTeamMateParams::default());
        let apo = steps.iter().find(|s| s.label.as_deref() == Some(labels::APOTHECARY_THROWN_PLAYER)).unwrap();
        assert_eq!(apo.step_id, StepId::Apothecary);
    }

    #[test]
    fn throw_team_mate_resolve_pass_is_labelled() {
        let steps = ThrowTeamMate::build_sequence(&ThrowTeamMateParams::default());
        let rp = steps.iter().find(|s| s.label.as_deref() == Some(labels::RESOLVE_PASS)).unwrap();
        assert_eq!(rp.step_id, StepId::DispatchScatterPlayer);
    }
}
