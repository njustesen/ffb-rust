/// BB2020 Throw Team-Mate step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2020.ThrowTeamMate`.
use ffb_model::enums::ApothecaryMode;
use ffb_model::types::FieldCoordinate;
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};

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
        let fl = labels::END_THROW_TEAM_MATE;

        // 1 INIT_THROW_TEAM_MATE
        let mut init_p = vec![
            StepParameter::GotoLabelOnEnd(fl.into()),
            StepParameter::IsKickedPlayer(params.is_kicked),
        ];
        if let Some(ref id) = params.thrown_player_id {
            init_p.push(StepParameter::ThrownPlayerId(Some(id.clone())));
        }
        if let Some(coord) = params.target_coordinate {
            init_p.push(StepParameter::TargetCoordinate(coord));
        }
        seq.add(StepId::InitThrowTeamMate, init_p);

        // 2-13 ACTIVATION BLOCK (with GotoLabel, SetDefender for thrown player, BloodLust → fl)
        let fl_s: String = fl.into();
        seq.add(StepId::InitActivation, vec![]);
        seq.add(StepId::AnimalSavagery, vec![StepParameter::GotoLabelOnFailure(fl_s.clone())]);
        seq.add(StepId::HandleDropPlayerContext, vec![]);
        seq.add(StepId::PlaceBall, vec![]);
        seq.add(StepId::Apothecary, vec![StepParameter::ApothecaryMode(ApothecaryMode::AnimalSavagery)]);
        seq.add(StepId::CatchScatterThrowIn, vec![]);
        // SetDefender with block_defender_id=thrown_player_id (unconditional, matches Java)
        seq.add(StepId::SetDefender, vec![StepParameter::BlockDefenderId(
            params.thrown_player_id.clone().unwrap_or_default(),
        )]);
        seq.add(StepId::GotoLabel, vec![
            StepParameter::GotoLabel(labels::NEXT.into()),
            StepParameter::AlternateGotoLabel(fl_s.clone()),
        ]);
        seq.add_labelled(StepId::BoneHead, labels::NEXT, vec![StepParameter::GotoLabelOnFailure(fl_s.clone())]);
        seq.add(StepId::ReallyStupid, vec![StepParameter::GotoLabelOnFailure(fl_s.clone())]);
        seq.add(StepId::TakeRoot, vec![]);
        seq.add(StepId::UnchannelledFury, vec![StepParameter::GotoLabelOnFailure(fl_s.clone())]);
        seq.add(StepId::BloodLust, vec![StepParameter::GotoLabelOnFailure(fl_s.clone())]);

        // ALWAYS_HUNGRY (failure → EAT_TEAM_MATE, success → RESOLVE_PASS)
        seq.add(StepId::AlwaysHungry, vec![
            StepParameter::IsKickedPlayer(params.is_kicked),
            StepParameter::GotoLabelOnFailure(labels::EAT_TEAM_MATE.into()),
            StepParameter::GotoLabelOnSuccess(labels::RESOLVE_PASS.into()),
        ]);
        // THROW_TEAM_MATE
        seq.add(StepId::ThrowTeamMate, vec![StepParameter::IsKickedPlayer(params.is_kicked)]);
        // DISPATCH_SCATTER_PLAYER [RESOLVE_PASS]
        seq.add_labelled(StepId::DispatchScatterPlayer, labels::RESOLVE_PASS, vec![
            StepParameter::IsKickedPlayer(params.is_kicked),
        ]);
        // RIGHT_STUFF [RIGHT_STUFF]
        seq.add_labelled(StepId::RightStuff, labels::RIGHT_STUFF, vec![
            StepParameter::IsKickedPlayer(params.is_kicked),
            StepParameter::GotoLabelOnSuccess(labels::END_SCATTER_PLAYER.into()),
        ]);
        // GOTO → APOTHECARY_THROWN_PLAYER
        seq.jump(labels::APOTHECARY_THROWN_PLAYER);
        // PICK_UP [END_SCATTER_PLAYER] (with SCATTER_BALL failure, ThrownPlayerId)
        let mut pickup_params = vec![StepParameter::GotoLabelOnFailure(labels::SCATTER_BALL.into())];
        if let Some(ref id) = params.thrown_player_id {
            pickup_params.push(StepParameter::ThrownPlayerId(Some(id.clone())));
        }
        seq.add_labelled(StepId::PickUp, labels::END_SCATTER_PLAYER, pickup_params);
        // GOTO → END_THROW_TEAM_MATE
        seq.jump(fl);
        // EAT_TEAM_MATE [EAT_TEAM_MATE]
        seq.add_labelled(StepId::EatTeamMate, labels::EAT_TEAM_MATE, vec![]);
        // APOTHECARY [APOTHECARY_THROWN_PLAYER] (ThrownPlayer)
        seq.add_labelled(StepId::Apothecary, labels::APOTHECARY_THROWN_PLAYER, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::ThrownPlayer),
        ]);
        // CATCH_SCATTER_THROW_IN [SCATTER_BALL]
        seq.add_labelled(StepId::CatchScatterThrowIn, labels::SCATTER_BALL, vec![]);
        // RESET_TO_MOVE [END_THROW_TEAM_MATE]
        seq.add_labelled(StepId::ResetToMove, fl, vec![]);
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
    fn throw_team_mate_ends_with_end_throw_team_mate() {
        let steps = ThrowTeamMate::build_sequence(&ThrowTeamMateParams::default());
        assert_eq!(steps.last().unwrap().step_id, StepId::EndThrowTeamMate);
    }

    #[test]
    fn throw_team_mate_has_activation_block() {
        let steps = ThrowTeamMate::build_sequence(&ThrowTeamMateParams::default());
        assert!(steps.iter().any(|s| s.step_id == StepId::InitActivation));
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
