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
    seq.extend(kickoff_tail(rules));
    seq
}

/// The parity harness's flattened kickoff tail.
///
/// BB2020 additionally needs the two GOTO labels Java's `generator/mixed/Kickoff` threads onto
/// APPLY_KICKOFF_RESULT, plus the labelled `BLITZ_TURN` / `KICKOFF_ANIMATION` / `END_KICKOFF`
/// targets and the jump that skips the blitz turn on the normal path — BB2020's kickoff table maps
/// roll 10 to BLITZ, whose handler does `GOTO_LABEL fGotoLabelOnBlitz`. Without them the goto target
/// is the empty string and the whole step stack drains.
fn kickoff_tail(rules: ffb_model::enums::Rules) -> Vec<SequenceStep> {
    use ffb_model::enums::Rules;
    let bb2020 = rules == Rules::Bb2020;
    let mut seq = vec![
        SequenceStep::new(StepId::CoinChoice),
        SequenceStep::new(StepId::ReceiveChoice),
        SequenceStep::new(StepId::InitKickoff),
        SequenceStep::new(StepId::Setup),
        SequenceStep::new(StepId::Setup),
        SequenceStep::new(StepId::Kickoff),
        SequenceStep::new(StepId::KickoffScatterRoll),
        SequenceStep::new(StepId::KickoffResultRoll),
        SequenceStep::new(StepId::ApplyKickoffResult),
        // Java `generator/mixed/Kickoff.java:45-46`:
        //     sequence.add(StepId.APOTHECARY, from(APOTHECARY_MODE, ApothecaryMode.HOME));
        //     sequence.add(StepId.APOTHECARY, from(APOTHECARY_MODE, ApothecaryMode.AWAY));
        // These consume the INJURY_RESULT that a kickoff event publishes for a hit player and APPLY
        // it. Without them the result was published into a sequence that had no consumer: the
        // Officious Ref's Ball & Chain injury was rolled and then dropped, leaving the goblin Fanatic
        // Standing where Java has it Ko (goblin bb2020 seed 38 i=1, `a02`).
        SequenceStep::with_params(StepId::Apothecary,
            vec![StepParameter::ApothecaryMode(ApothecaryMode::Home)]),
        SequenceStep::with_params(StepId::Apothecary,
            vec![StepParameter::ApothecaryMode(ApothecaryMode::Away)]),
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
    ];
    if bb2020 {
        add_bb2020_kickoff_labels(&mut seq);
    }
    seq
}

/// Give a flattened kickoff sequence the labels Java's `generator/mixed/Kickoff.pushSequence`
/// threads, so BB2020's `BLITZ` (roll 10) and the `GOTO_LABEL_ON_END` fall-through have somewhere
/// to jump:
///
/// ```java
/// sequence.add(APPLY_KICKOFF_RESULT, from(GOTO_LABEL_ON_END, END_KICKOFF),
///                                    from(GOTO_LABEL_ON_BLITZ, BLITZ_TURN));
/// ...
/// sequence.jump(KICKOFF_ANIMATION);
/// sequence.add(BLITZ_TURN, IStepLabel.BLITZ_TURN);
/// sequence.add(KICKOFF_ANIMATION, IStepLabel.KICKOFF_ANIMATION);
/// ...
/// sequence.add(END_KICKOFF, IStepLabel.END_KICKOFF);
/// ```
///
/// Without them the goto target is the empty string and the whole step stack drains — the game ends
/// mid-drive (bb2020 human seed 23 at the opening kickoff, seed 38 at the half-2 kickoff).
fn add_bb2020_kickoff_labels(seq: &mut Vec<SequenceStep>) {
    use crate::step::generator::sequence::labels;
    if let Some(akr) = seq.iter_mut().find(|st| st.step_id == StepId::ApplyKickoffResult) {
        akr.params.push(StepParameter::GotoLabelOnEnd(labels::END_KICKOFF.into()));
        akr.params.push(StepParameter::GotoLabelOnBlitz(labels::BLITZ_TURN.into()));
    }
    let anim = match seq.iter().position(|st| st.step_id == StepId::KickoffAnimation) {
        Some(i) => i,
        None => return,
    };
    // The jump target must itself be labelled, or the GotoLabel drains the stack in turn.
    seq[anim].label = Some(labels::KICKOFF_ANIMATION.into());
    seq.insert(anim, SequenceStep::labelled(StepId::BlitzTurn, labels::BLITZ_TURN, Vec::new()));
    seq.insert(anim, SequenceStep::with_params(StepId::GotoLabel,
        vec![StepParameter::GotoLabel(labels::KICKOFF_ANIMATION.into())]));
    if let Some(last) = seq.last_mut() {
        last.label = Some(labels::END_KICKOFF.into());
    }
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

/// The half-2 / post-touchdown kickoff. BB2020 needs the same labels as `kickoff_tail` — its
/// kickoff table can roll BLITZ here too (bb2020 human seed 38 died at the half-2 kickoff).
pub fn h2_kickoff_sequence_for(rules: ffb_model::enums::Rules) -> Vec<SequenceStep> {
    let mut seq = h2_kickoff_sequence();
    if rules == ffb_model::enums::Rules::Bb2020 {
        add_bb2020_kickoff_labels(&mut seq);
    }
    seq
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

    /// Java `generator/mixed/Kickoff.java:45-46` places APOTHECARY(HOME) and APOTHECARY(AWAY)
    /// immediately after APPLY_KICKOFF_RESULT. They consume and APPLY the INJURY_RESULT a kickoff
    /// event publishes for a hit player; without them the Officious Ref's Ball & Chain injury was
    /// rolled and then silently dropped (goblin bb2020 seed 38: the Fanatic stayed Standing where
    /// Java has it Ko).
    #[test]
    fn kickoff_sequence_applies_kickoff_event_injuries() {
        use ffb_model::enums::Rules;
        for rules in [Rules::Bb2016, Rules::Bb2020, Rules::Bb2025] {
            let seq = kickoff_tail(rules);
            let akr = seq.iter().position(|st| st.step_id == StepId::ApplyKickoffResult)
                .unwrap_or_else(|| panic!("{rules:?}: no ApplyKickoffResult"));
            let modes: Vec<ApothecaryMode> = seq[akr + 1..].iter()
                .take_while(|st| st.step_id == StepId::Apothecary)
                .filter_map(|st| st.params.iter().find_map(|p| match p {
                    StepParameter::ApothecaryMode(m) => Some(*m),
                    _ => None,
                }))
                .collect();
            assert_eq!(modes, vec![ApothecaryMode::Home, ApothecaryMode::Away],
                "{rules:?}: APPLY_KICKOFF_RESULT must be followed by APOTHECARY(HOME) then AWAY");
        }
    }
    /// Java's `generator/mixed/Kickoff.pushSequence` threads GOTO_LABEL_ON_END/ON_BLITZ onto
    /// APPLY_KICKOFF_RESULT and labels BLITZ_TURN / KICKOFF_ANIMATION / END_KICKOFF. BB2020's
    /// kickoff table maps roll 10 to BLITZ, whose handler GOTOs the blitz label, so a flattened
    /// BB2020 kickoff sequence without those labels drains the entire step stack and ends the game
    /// mid-drive — seed 23 at the opening kickoff, seed 38 at the half-2 one. Both entry points
    /// must therefore carry them, and no other edition may be affected.
    #[test]
    fn bb2020_kickoff_sequences_carry_the_blitz_labels() {
        use ffb_model::enums::Rules;
        use crate::step::generator::sequence::labels;

        for seq in [
            start_game_sequence_for(Rules::Bb2020),
            h2_kickoff_sequence_for(Rules::Bb2020),
        ] {
            let akr = seq.iter().find(|st| st.step_id == StepId::ApplyKickoffResult)
                .expect("sequence contains ApplyKickoffResult");
            assert!(akr.params.iter().any(|p|
                matches!(p, StepParameter::GotoLabelOnBlitz(l) if l == labels::BLITZ_TURN)),
                "ApplyKickoffResult must carry GotoLabelOnBlitz");
            assert!(akr.params.iter().any(|p|
                matches!(p, StepParameter::GotoLabelOnEnd(l) if l == labels::END_KICKOFF)),
                "ApplyKickoffResult must carry GotoLabelOnEnd");
            // Every goto target the step can name must exist as a label in the same sequence.
            for want in [labels::BLITZ_TURN, labels::KICKOFF_ANIMATION, labels::END_KICKOFF] {
                assert!(seq.iter().any(|st| st.label.as_deref() == Some(want)),
                    "missing labelled target {want}");
            }
            // The blitz turn must be skipped on the normal path (Java: `sequence.jump(...)`).
            let jump = seq.iter().position(|st| st.step_id == StepId::GotoLabel)
                .expect("sequence jumps past BLITZ_TURN");
            let blitz = seq.iter().position(|st| st.label.as_deref() == Some(labels::BLITZ_TURN))
                .expect("sequence has a labelled BlitzTurn");
            assert!(jump < blitz, "the jump must precede the labelled BLITZ_TURN step");
        }
    }

    /// The other editions keep the lighter flattened sequences untouched: BB2025 has no BLITZ
    /// kickoff result, and BB2016 reroutes through StepSpectators' full generator.
    #[test]
    fn non_bb2020_kickoff_sequences_are_unchanged() {
        use ffb_model::enums::Rules;
        for rules in [Rules::Bb2016, Rules::Bb2025] {
            for seq in [start_game_sequence_for(rules), h2_kickoff_sequence_for(rules)] {
                assert!(!seq.iter().any(|st| st.step_id == StepId::BlitzTurn),
                    "{rules:?} must not gain a BlitzTurn step");
                let akr = seq.iter().find(|st| st.step_id == StepId::ApplyKickoffResult).unwrap();
                assert!(akr.params.is_empty(), "{rules:?} ApplyKickoffResult must stay bare");
            }
        }
    }

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
