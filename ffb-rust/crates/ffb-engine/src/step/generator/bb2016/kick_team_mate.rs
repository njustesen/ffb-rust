/// BB2016 kick-team-mate step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2016.KickTeamMate`.
use ffb_model::enums::ApothecaryMode;
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};

/// Parameters for the BB2016 KickTeamMate sequence.
#[derive(Debug, Clone, Default)]
pub struct KickTeamMateParams {
    pub kicked_player_id: Option<String>,
    pub num_dice: i32,
}

pub struct KickTeamMate;

impl KickTeamMate {
    pub fn new() -> Self { Self }

    /// Build the BB2016 kick-team-mate step sequence (Java `pushSequence`).
    pub fn build_sequence(params: &KickTeamMateParams) -> Vec<SequenceStep> {
        let mut seq = Sequence::new();
        let fl = labels::END_KICK_TEAM_MATE;

        // 1 INIT_KICK_TEAM_MATE
        let mut init_params = vec![
            StepParameter::GotoLabelOnEnd(fl.into()),
            StepParameter::NrOfDice(params.num_dice),
        ];
        if let Some(ref id) = params.kicked_player_id {
            init_params.push(StepParameter::KickedPlayerId(Some(id.clone())));
        }
        seq.add(StepId::InitKickTeamMate, init_params);

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
        // 7 KICK_TEAM_MATE
        seq.add(StepId::KickTeamMate, vec![
            StepParameter::GotoLabelOnFailure(labels::KICK_TM_DOUBLE_ROLLED.into()),
        ]);
        // 8 GOTO_LABEL → RIGHT_STUFF (insert scatterPlayerSequence here at runtime)
        seq.jump(labels::RIGHT_STUFF);
        // 9 KICK_TM_DOUBLE_ROLLED [KICK_TM_DOUBLE_ROLLED]
        seq.add_labelled(StepId::KickTeamMateDoubleRolled, labels::KICK_TM_DOUBLE_ROLLED, vec![]);
        // 10 GOTO_LABEL → APOTHECARY_KICKED_PLAYER
        seq.jump(labels::APOTHECARY_KICKED_PLAYER);
        // 11 RIGHT_STUFF [RIGHT_STUFF]
        seq.add_labelled(StepId::RightStuff, labels::RIGHT_STUFF, vec![]);
        // 12 GOTO_LABEL → APOTHECARY_KICKED_PLAYER
        seq.jump(labels::APOTHECARY_KICKED_PLAYER);
        // 13 APOTHECARY [APOTHECARY_KICKED_PLAYER] (THROWN_PLAYER)
        seq.add_labelled(StepId::Apothecary, labels::APOTHECARY_KICKED_PLAYER, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::ThrownPlayer),
        ]);
        // 14 CATCH_SCATTER_THROW_IN
        seq.add(StepId::CatchScatterThrowIn, vec![]);
        // 15 END_KICK_TEAM_MATE [END_KICK_TEAM_MATE]
        seq.add_labelled(StepId::EndKickTeamMate, fl, vec![]);

        seq.build()
    }
}

impl Default for KickTeamMate {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kick_team_mate_starts_with_init() {
        let steps = KickTeamMate::build_sequence(&KickTeamMateParams::default());
        assert_eq!(steps[0].step_id, StepId::InitKickTeamMate);
    }

    #[test]
    fn kick_team_mate_ends_with_end_labelled() {
        let steps = KickTeamMate::build_sequence(&KickTeamMateParams::default());
        let last = steps.last().unwrap();
        assert_eq!(last.step_id, StepId::EndKickTeamMate);
        assert_eq!(last.label.as_deref(), Some(labels::END_KICK_TEAM_MATE));
    }

    #[test]
    fn kick_team_mate_has_kick_tm_double_rolled_labelled() {
        let steps = KickTeamMate::build_sequence(&KickTeamMateParams::default());
        let ktdr = steps.iter().find(|s| s.label.as_deref() == Some(labels::KICK_TM_DOUBLE_ROLLED)).unwrap();
        assert_eq!(ktdr.step_id, StepId::KickTeamMateDoubleRolled);
    }

    #[test]
    fn kick_team_mate_has_apothecary_kicked_player_labelled() {
        let steps = KickTeamMate::build_sequence(&KickTeamMateParams::default());
        let apo = steps.iter().find(|s| s.label.as_deref() == Some(labels::APOTHECARY_KICKED_PLAYER)).unwrap();
        assert_eq!(apo.step_id, StepId::Apothecary);
    }

    #[test]
    fn kick_team_mate_right_stuff_is_labelled() {
        let steps = KickTeamMate::build_sequence(&KickTeamMateParams::default());
        let rs = steps.iter().find(|s| s.step_id == StepId::RightStuff && s.label.is_some()).unwrap();
        assert_eq!(rs.label.as_deref(), Some(labels::RIGHT_STUFF));
    }
}
