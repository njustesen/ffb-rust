/// BB2016 Throw Team-Mate step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2016.ThrowTeamMate`.
use ffb_model::enums::ApothecaryMode;
use ffb_model::types::FieldCoordinate;
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};

/// Parameters for the BB2016 ThrowTeamMate sequence.
#[derive(Debug, Clone, Default)]
pub struct ThrowTeamMateParams {
    pub thrown_player_id: Option<String>,
    pub target_coordinate: Option<FieldCoordinate>,
}

pub struct ThrowTeamMate;

impl ThrowTeamMate {
    pub fn new() -> Self { Self }

    /// Build the BB2016 throw-team-mate step sequence (Java `pushSequence`).
    pub fn build_sequence(params: &ThrowTeamMateParams) -> Vec<SequenceStep> {
        let mut seq = Sequence::new();
        let fl = labels::END_THROW_TEAM_MATE;

        // 1 INIT_THROW_TEAM_MATE
        let mut init_params = vec![StepParameter::GotoLabelOnEnd(fl.into())];
        if let Some(ref id) = params.thrown_player_id {
            init_params.push(StepParameter::ThrownPlayerId(Some(id.clone())));
        }
        if let Some(coord) = params.target_coordinate {
            init_params.push(StepParameter::TargetCoordinate(coord));
        }
        seq.add(StepId::InitThrowTeamMate, init_params);

        // 2 BONE_HEAD
        seq.add(StepId::BoneHead, vec![StepParameter::GotoLabelOnFailure(fl.into())]);
        // 3 REALLY_STUPID
        seq.add(StepId::ReallyStupid, vec![StepParameter::GotoLabelOnFailure(fl.into())]);
        // 4 TAKE_ROOT
        seq.add(StepId::TakeRoot, vec![]);
        // 5 WILD_ANIMAL
        seq.add(StepId::WildAnimal, vec![StepParameter::GotoLabelOnFailure(fl.into())]);
        // 6 BLOOD_LUST (no failure label)
        seq.add(StepId::BloodLust, vec![]);
        // 7 ALWAYS_HUNGRY
        seq.add(StepId::AlwaysHungry, vec![
            StepParameter::GotoLabelOnFailure(labels::EAT_TEAM_MATE.into()),
            StepParameter::GotoLabelOnSuccess(labels::FUMBLE_TTM_PASS.into()),
        ]);
        // 8 THROW_TEAM_MATE
        seq.add(StepId::ThrowTeamMate, vec![
            StepParameter::GotoLabelOnFailure(labels::FUMBLE_TTM_PASS.into()),
        ]);
        // 9 GOTO_LABEL → RIGHT_STUFF (insert scatterPlayerSequence here at runtime)
        seq.jump(labels::RIGHT_STUFF);
        // 10 FUMBLE_TTM_PASS [FUMBLE_TTM_PASS]
        seq.add_labelled(StepId::FumbleTtmPass, labels::FUMBLE_TTM_PASS, vec![]);
        // 11 RIGHT_STUFF [RIGHT_STUFF]
        seq.add_labelled(StepId::RightStuff, labels::RIGHT_STUFF, vec![]);
        // 12 GOTO_LABEL → APOTHECARY_THROWN_PLAYER
        seq.jump(labels::APOTHECARY_THROWN_PLAYER);
        // 13 EAT_TEAM_MATE [EAT_TEAM_MATE]
        seq.add_labelled(StepId::EatTeamMate, labels::EAT_TEAM_MATE, vec![]);
        // 14 APOTHECARY [APOTHECARY_THROWN_PLAYER] (THROWN_PLAYER)
        seq.add_labelled(StepId::Apothecary, labels::APOTHECARY_THROWN_PLAYER, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::ThrownPlayer),
        ]);
        // 15 CATCH_SCATTER_THROW_IN
        seq.add(StepId::CatchScatterThrowIn, vec![]);
        // 16 END_THROW_TEAM_MATE [END_THROW_TEAM_MATE]
        seq.add_labelled(StepId::EndThrowTeamMate, fl, vec![]);

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
    fn throw_team_mate_starts_with_init() {
        let steps = ThrowTeamMate::build_sequence(&ThrowTeamMateParams::default());
        assert_eq!(steps[0].step_id, StepId::InitThrowTeamMate);
    }

    #[test]
    fn throw_team_mate_ends_with_end_throw_team_mate_labelled() {
        let steps = ThrowTeamMate::build_sequence(&ThrowTeamMateParams::default());
        let last = steps.last().unwrap();
        assert_eq!(last.step_id, StepId::EndThrowTeamMate);
        assert_eq!(last.label.as_deref(), Some(labels::END_THROW_TEAM_MATE));
    }

    #[test]
    fn throw_team_mate_fumble_ttm_pass_is_labelled() {
        let steps = ThrowTeamMate::build_sequence(&ThrowTeamMateParams::default());
        let ftp = steps.iter().find(|s| s.label.as_deref() == Some(labels::FUMBLE_TTM_PASS)).unwrap();
        assert_eq!(ftp.step_id, StepId::FumbleTtmPass);
    }

    #[test]
    fn throw_team_mate_eat_team_mate_is_labelled() {
        let steps = ThrowTeamMate::build_sequence(&ThrowTeamMateParams::default());
        let etm = steps.iter().find(|s| s.label.as_deref() == Some(labels::EAT_TEAM_MATE)).unwrap();
        assert_eq!(etm.step_id, StepId::EatTeamMate);
    }

    #[test]
    fn throw_team_mate_apothecary_thrown_player_is_labelled() {
        let steps = ThrowTeamMate::build_sequence(&ThrowTeamMateParams::default());
        let apo = steps.iter().find(|s| s.label.as_deref() == Some(labels::APOTHECARY_THROWN_PLAYER)).unwrap();
        assert_eq!(apo.step_id, StepId::Apothecary);
    }
}
