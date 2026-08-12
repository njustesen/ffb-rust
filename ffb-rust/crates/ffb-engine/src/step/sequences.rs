//! BB2025 step sequences — 1:1 port of Java generator classes.
//! Each function returns a `Vec<SequenceStep>` that the driver pushes onto the step stack.
use ffb_model::enums::{ApothecaryMode, InducementPhase};
use super::framework::{StepId, SequenceStep, StepParameter};

pub fn start_game_sequence() -> Vec<SequenceStep> {
    start_game_sequence_for(ffb_model::enums::Rules::Bb2025)
}

/// Edition-aware parity pregame. BB2016's Java `StartGame` runs WEATHER *before* SPECTATORS
/// (bb2016/StartGame.java), the reverse of BB2020+/mixed. Both then flow into the same flattened
/// kickoff steps (CoinChoice → ReceiveChoice → …). The per-team dice inside SPECTATORS also differ
/// (2D6 bb2016 vs 1D3 bb2020+) — routed by `make_step_for`, not here.
pub fn start_game_sequence_for(rules: ffb_model::enums::Rules) -> Vec<SequenceStep> {
    use ffb_model::enums::Rules;
    let pregame: Vec<SequenceStep> = if rules == Rules::Bb2016 {
        vec![
            SequenceStep::new(StepId::InitStartGame),
            SequenceStep::new(StepId::Weather),
            SequenceStep::new(StepId::Spectators),
        ]
    } else {
        vec![
            SequenceStep::new(StepId::InitStartGame),
            SequenceStep::new(StepId::Spectators),
            SequenceStep::new(StepId::Weather),
        ]
    };
    let mut seq = pregame;
    seq.extend(kickoff_tail());
    seq
}

fn kickoff_tail() -> Vec<SequenceStep> {
    vec![
        SequenceStep::new(StepId::CoinChoice),
        SequenceStep::new(StepId::ReceiveChoice),
        SequenceStep::new(StepId::InitKickoff),
        SequenceStep::new(StepId::Setup),
        SequenceStep::new(StepId::Setup),
        SequenceStep::new(StepId::Kickoff),
        SequenceStep::new(StepId::KickoffScatterRoll),
        SequenceStep::new(StepId::KickoffResultRoll),
        SequenceStep::new(StepId::ApplyKickoffResult),
        // Java Kickoff generator: KICKOFF_ANIMATION between APPLY_KICKOFF_RESULT and
        // CATCH_SCATTER_THROW_IN — it sets ballInPlay(true) and publishes the CATCH_KICKOFF
        // mode. Omitting it left ball_in_play false for the entire drive, which silently
        // disabled every pickup/catch roll (StepPickUp guards on ball_in_play).
        SequenceStep::new(StepId::KickoffAnimation),
        SequenceStep::new(StepId::CatchScatterThrowIn),
        SequenceStep::new(StepId::Touchback),
        // Java: a second CATCH_SCATTER_THROW_IN resolves the post-touchback ball placement.
        SequenceStep::new(StepId::CatchScatterThrowIn),
        SequenceStep::new(StepId::EndKickoff),
    ]
}

/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2025.EndTurn.pushSequence`.
/// `check_forgo` is forwarded to StepForgoneStalling (true only when the acting player
/// may have stalled their activation intentionally).
pub fn end_turn_sequence(check_forgo: bool) -> Vec<SequenceStep> {
    vec![
        SequenceStep::with_params(StepId::ForgoneStalling, vec![StepParameter::CheckForgo(check_forgo)]),
        SequenceStep::with_params(StepId::SteadyFooting, vec![StepParameter::ApothecaryMode(ApothecaryMode::HitPlayer)]),
        SequenceStep::new(StepId::PlaceBall),
        SequenceStep::with_params(StepId::Apothecary, vec![StepParameter::ApothecaryMode(ApothecaryMode::HitPlayer)]),
        SequenceStep::new(StepId::CatchScatterThrowIn),
        SequenceStep::new(StepId::EndTurn),
    ]
}

/// Mirrors Java `com.fumbbl.ffb.server.step.generator.bb2025.EndGame.pushSequence`.
pub fn end_game_sequence(admin_mode: bool) -> Vec<SequenceStep> {
    vec![
        SequenceStep::with_params(StepId::InitEndGame, vec![
            StepParameter::GotoLabelOnEnd("END_GAME".into()),
            StepParameter::AdminMode(admin_mode),
        ]),
        SequenceStep::new(StepId::PenaltyShootout),
        SequenceStep::new(StepId::Mvp),
        SequenceStep::new(StepId::Winnings),
        SequenceStep::new(StepId::DedicatedFans),
        SequenceStep::new(StepId::PlayerLoss),
        SequenceStep::labelled(StepId::EndGame, "END_GAME", vec![]),
    ]
}

pub fn h2_kickoff_sequence() -> Vec<SequenceStep> {
    vec![
        SequenceStep::new(StepId::InitKickoff),
        SequenceStep::new(StepId::Setup),
        SequenceStep::new(StepId::Setup),
        SequenceStep::new(StepId::Kickoff),
        SequenceStep::new(StepId::KickoffScatterRoll),
        SequenceStep::new(StepId::KickoffResultRoll),
        SequenceStep::new(StepId::ApplyKickoffResult),
        // See start_game_sequence: KICKOFF_ANIMATION sets ballInPlay(true) (pickups/catches
        // are dead without it), and the touchback resolves via a second CST step.
        SequenceStep::new(StepId::KickoffAnimation),
        SequenceStep::new(StepId::CatchScatterThrowIn),
        SequenceStep::new(StepId::Touchback),
        SequenceStep::new(StepId::CatchScatterThrowIn),
        SequenceStep::new(StepId::EndKickoff),
    ]
}

pub fn move_sequence() -> Vec<SequenceStep> {
    vec![
        SequenceStep::new(StepId::InitMoving),
        SequenceStep::new(StepId::PickUp),
        SequenceStep::new(StepId::EndMoving),
        SequenceStep::new(StepId::EndPlayerAction),
    ]
}

pub fn blitz_sequence() -> Vec<SequenceStep> {
    vec![
        SequenceStep::new(StepId::BlockRoll),
        SequenceStep::new(StepId::EndPlayerAction),
    ]
}

pub fn block_sequence() -> Vec<SequenceStep> {
    vec![
        SequenceStep::new(StepId::BlockRoll),
        SequenceStep::new(StepId::EndPlayerAction),
    ]
}

pub fn standup_end_sequence() -> Vec<SequenceStep> {
    vec![
        SequenceStep::new(StepId::EndPlayerAction),
    ]
}

pub fn pass_sequence() -> Vec<SequenceStep> {
    vec![
        SequenceStep::new(StepId::PickUp),
        SequenceStep::new(StepId::Intercept),
        SequenceStep::new(StepId::Pass),
        SequenceStep::new(StepId::EndPlayerAction),
    ]
}

pub fn handoff_sequence() -> Vec<SequenceStep> {
    vec![
        SequenceStep::new(StepId::HandOver),
        SequenceStep::new(StepId::EndPlayerAction),
    ]
}

pub fn foul_sequence() -> Vec<SequenceStep> {
    vec![
        SequenceStep::new(StepId::Foul),
        SequenceStep::new(StepId::EndPlayerAction),
    ]
}

pub fn standup_blitz_sequence() -> Vec<SequenceStep> {
    vec![
        SequenceStep::new(StepId::BlockRoll),
        SequenceStep::new(StepId::EndPlayerAction),
    ]
}

/// Mirrors Java `com.fumbbl.ffb.server.step.generator.common.Inducement.pushSequence`
/// with the 3-arg `SequenceParams` constructor (`checkForgo` defaults to `false`).
pub fn inducement_sequence(phase: InducementPhase, home_team: bool) -> Vec<SequenceStep> {
    inducement_sequence_with_check_forgo(phase, home_team, false)
}

/// Mirrors Java `com.fumbbl.ffb.server.step.generator.common.Inducement.pushSequence`
/// with the 4-arg `SequenceParams` constructor, forwarding an explicit `checkForgo` value
/// to the `END_INDUCEMENT` step (used by `StepEndFeeding`'s end-of-opponent-turn call).
pub fn inducement_sequence_with_check_forgo(phase: InducementPhase, home_team: bool, check_forgo: bool) -> Vec<SequenceStep> {
    vec![
        SequenceStep::with_params(StepId::InitInducement, vec![
            StepParameter::InducementPhase(phase),
            StepParameter::HomeTeam(home_team),
        ]),
        SequenceStep::with_params(StepId::Apothecary, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::Defender),
        ]),
        SequenceStep::with_params(StepId::Apothecary, vec![
            StepParameter::ApothecaryMode(ApothecaryMode::Attacker),
        ]),
        SequenceStep::new(StepId::CatchScatterThrowIn),
        SequenceStep::with_params(StepId::EndInducement, vec![
            StepParameter::CheckForgo(check_forgo),
        ]),
    ]
}

/// Java: HitAndRun / ball-enters-square sequence — PickUp followed by CatchScatterThrowIn.
pub fn pick_up_catch_scatter_sequence() -> Vec<SequenceStep> {
    vec![
        SequenceStep::new(StepId::PickUp),
        SequenceStep::new(StepId::CatchScatterThrowIn),
    ]
}

pub fn select_sequence() -> Vec<SequenceStep> {
    let mut seq = Vec::with_capacity(20);
    seq.push(SequenceStep::new(StepId::InitSelecting));
    for _ in 0..14 {
        seq.push(SequenceStep::new(StepId::NoOp));
    }
    for _ in 0..4 {
        seq.push(SequenceStep::new(StepId::NoOp));
    }
    seq.push(SequenceStep::labelled(StepId::EndSelecting, "END_SELECTING", vec![]));
    seq
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_game_sequence_starts_with_init_start_game() {
        let seq = start_game_sequence();
        assert_eq!(seq[0].step_id, StepId::InitStartGame);
    }

    #[test]
    fn end_turn_sequence_ends_with_end_turn() {
        let seq = end_turn_sequence(false);
        assert_eq!(seq.last().unwrap().step_id, StepId::EndTurn);
    }

    #[test]
    fn bb2016_pregame_rolls_weather_before_spectators() {
        // Java bb2016/StartGame: WEATHER then SPECTATORS (reverse of bb2020+/mixed).
        use ffb_model::enums::Rules;
        let seq = start_game_sequence_for(Rules::Bb2016);
        let w = seq.iter().position(|s| s.step_id == StepId::Weather).unwrap();
        let s = seq.iter().position(|s| s.step_id == StepId::Spectators).unwrap();
        assert!(w < s, "bb2016 must run Weather before Spectators, got w={w} s={s}");
    }

    #[test]
    fn bb2025_pregame_rolls_spectators_before_weather() {
        use ffb_model::enums::Rules;
        let seq = start_game_sequence_for(Rules::Bb2025);
        let w = seq.iter().position(|s| s.step_id == StepId::Weather).unwrap();
        let s = seq.iter().position(|s| s.step_id == StepId::Spectators).unwrap();
        assert!(s < w, "bb2025 must run Spectators before Weather");
    }

    #[test]
    fn end_turn_sequence_check_forgo_param_propagates() {
        let seq = end_turn_sequence(true);
        assert!(matches!(seq[0].params[0], StepParameter::CheckForgo(true)));
    }

    #[test]
    fn inducement_sequence_carries_phase_param() {
        let seq = inducement_sequence(InducementPhase::AfterKickoffToOpponent, true);
        assert!(matches!(seq[0].params[0], StepParameter::InducementPhase(InducementPhase::AfterKickoffToOpponent)));
    }

    #[test]
    fn select_sequence_last_step_has_end_selecting_label() {
        let seq = select_sequence();
        let last = seq.last().unwrap();
        assert_eq!(last.step_id, StepId::EndSelecting);
        assert_eq!(last.label.as_deref(), Some("END_SELECTING"));
    }
}
