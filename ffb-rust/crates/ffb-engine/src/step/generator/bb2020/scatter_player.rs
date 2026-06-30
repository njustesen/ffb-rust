/// BB2020 scatter-player step sequence.
/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2020.ScatterPlayer`.
use ffb_model::enums::ApothecaryMode;
use ffb_model::types::FieldCoordinate;
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::sequence::{Sequence, SequenceStep, labels};

#[derive(Debug, Clone, Default)]
pub struct ScatterPlayerParams {
    pub thrown_player_id: Option<String>,
    pub thrown_player_state: Option<ffb_model::enums::PlayerState>,
    pub thrown_player_has_ball: bool,
    pub thrown_player_coordinate: Option<FieldCoordinate>,
    pub throw_scatter: bool,
    pub has_swoop: bool,
    pub deviates: bool,
    pub crash_landing: bool,
}

pub struct ScatterPlayer;

impl ScatterPlayer {
    pub fn new() -> Self { Self }

    pub fn build_sequence(params: &ScatterPlayerParams) -> Vec<SequenceStep> {
        let mut seq = Sequence::new();

        // Optional leading SWOOP step
        if params.has_swoop {
            let mut swoop_params = vec![
                StepParameter::ThrownPlayerHasBall(params.thrown_player_has_ball),
                StepParameter::ThrowScatter(params.throw_scatter),
                StepParameter::GotoLabelOnFallDown(labels::APOTHECARY_HIT_PLAYER.into()),
            ];
            if let Some(ref id) = params.thrown_player_id {
                swoop_params.push(StepParameter::ThrownPlayerId(Some(id.clone())));
            }
            if let Some(state) = params.thrown_player_state {
                swoop_params.push(StepParameter::ThrownPlayerState(state));
            }
            if let Some(coord) = params.thrown_player_coordinate {
                swoop_params.push(StepParameter::ThrownPlayerCoordinate(Some(coord)));
            }
            seq.add(StepId::Swoop, swoop_params);
        }

        // INIT_SCATTER_PLAYER
        let mut init_params = vec![
            StepParameter::ThrownPlayerHasBall(params.thrown_player_has_ball),
            StepParameter::ThrowScatter(params.throw_scatter),
            StepParameter::PassDeviates(params.deviates),
            StepParameter::CrashLanding(params.crash_landing),
        ];
        if let Some(ref id) = params.thrown_player_id {
            init_params.push(StepParameter::ThrownPlayerId(Some(id.clone())));
        }
        if let Some(state) = params.thrown_player_state {
            init_params.push(StepParameter::ThrownPlayerState(state));
        }
        if let Some(coord) = params.thrown_player_coordinate {
            init_params.push(StepParameter::ThrownPlayerCoordinate(Some(coord)));
        }
        seq.add(StepId::InitScatterPlayer, init_params);

        // TRAP_DOOR [SCATTER_BALL]
        seq.add_labelled(StepId::TrapDoor, labels::SCATTER_BALL, vec![]);
        // APOTHECARY (TrapDoor)
        seq.add(StepId::Apothecary, vec![StepParameter::ApothecaryMode(ApothecaryMode::TrapDoor)]);
        // NO SteadyFooting in BB2020
        // PLACE_BALL
        seq.add(StepId::PlaceBall, vec![]);
        // APOTHECARY [APOTHECARY_HIT_PLAYER] (HitPlayer)
        seq.add_labelled(StepId::Apothecary, labels::APOTHECARY_HIT_PLAYER, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::HitPlayer),
        ]);
        // CATCH_SCATTER_THROW_IN
        seq.add(StepId::CatchScatterThrowIn, vec![]);
        // END_SCATTER_PLAYER (no label — Java has no label here)
        seq.add(StepId::EndScatterPlayer, vec![]);

        seq.build()
    }
}

impl Default for ScatterPlayer {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scatter_player_without_swoop_has_7_steps() {
        let steps = ScatterPlayer::build_sequence(&ScatterPlayerParams::default());
        assert_eq!(steps.len(), 7);
    }

    #[test]
    fn scatter_player_with_swoop_has_8_steps() {
        let steps = ScatterPlayer::build_sequence(&ScatterPlayerParams { has_swoop: true, ..Default::default() });
        assert_eq!(steps.len(), 8);
    }

    #[test]
    fn scatter_player_ends_with_end_scatter_player_no_label() {
        let steps = ScatterPlayer::build_sequence(&ScatterPlayerParams::default());
        let last = steps.last().unwrap();
        assert_eq!(last.step_id, StepId::EndScatterPlayer);
        assert!(last.label.is_none());
    }

    #[test]
    fn scatter_player_has_no_steady_footing() {
        let steps = ScatterPlayer::build_sequence(&ScatterPlayerParams::default());
        assert!(!steps.iter().any(|s| s.step_id == StepId::SteadyFooting));
    }

    #[test]
    fn scatter_player_apothecary_hit_player_is_labelled() {
        let steps = ScatterPlayer::build_sequence(&ScatterPlayerParams::default());
        let apo = steps.iter().find(|s| s.label.as_deref() == Some(labels::APOTHECARY_HIT_PLAYER)).unwrap();
        assert_eq!(apo.step_id, StepId::Apothecary);
    }

    #[test]
    fn scatter_player_init_has_pass_deviates_and_crash_landing() {
        let params = ScatterPlayerParams { deviates: true, crash_landing: true, ..Default::default() };
        let steps = ScatterPlayer::build_sequence(&params);
        let init = steps.iter().find(|s| s.step_id == StepId::InitScatterPlayer).unwrap();
        assert!(init.params.iter().any(|p| matches!(p, StepParameter::PassDeviates(_))));
        assert!(init.params.iter().any(|p| matches!(p, StepParameter::CrashLanding(_))));
    }
}
