use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::enums::PlayerState;
use ffb_model::factory::serious_injury_factory::{AnySeriousInjury, SeriousInjuryFactory};
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::model::serious_injury::SeriousInjury as SeriousInjuryTrait;
use ffb_model::report::mixed::report_injury::ReportInjury;
use ffb_model::report::report_id::ReportId;
use ffb_model::report::skip_injury_parts::SkipInjuryParts;

/// 1:1 translation of `InjuryMessage.java` (bb2025).
///
/// `ReportInjury` on the Rust side already flattens `ArmorModifier`/`InjuryModifier`/
/// `CasualtyModifier` objects down to plain name strings (see
/// `crates/ffb-model/src/report/mixed/report_injury.rs`), matching the same
/// simplification `ReportSkillRoll` uses for roll modifiers elsewhere in this codebase.
/// That means the per-modifier sign/magnitude math (`armorModifierTotal`,
/// `injuryModifierTotal`, casualty `" + reportString() = total"`), the Claws/cancelling-AV
/// detection (`isRegisteredToSkillWithProperty`), and the Thick Skull/Stunty detection all
/// have no data to draw on here — each is called out with a `// java:` comment at its
/// original call site below rather than fabricated.
pub struct InjuryMessage;

/// Java: `PlayerState.getDescription()` — not yet ported onto the shared `PlayerState`
/// bitmask type, so it is translated locally here (matches `PlayerState.java` verbatim).
fn player_state_description(state: PlayerState) -> Option<&'static str> {
    use ffb_model::enums::*;
    match state.base() {
        b if b == PS_UNKNOWN => Some("is unknown"),
        b if b == PS_STANDING => Some("is standing"),
        b if b == PS_MOVING => Some("is moving"),
        b if b == PS_PRONE => Some("is prone"),
        b if b == PS_STUNNED => Some("has been stunned"),
        b if b == PS_KNOCKED_OUT => Some("has been knocked out"),
        b if b == PS_BADLY_HURT => Some("has been badly hurt"),
        b if b == PS_SERIOUS_INJURY => Some("has been seriously injured"),
        b if b == PS_RIP => Some("has been killed"),
        b if b == PS_RESERVE => Some("is in reserve"),
        b if b == PS_MISSING => Some("is missing the game"),
        b if b == PS_FALLING => Some("is about to fall down"),
        b if b == PS_BLOCKED => Some("is being blocked"),
        b if b == PS_BANNED => Some("is banned from the game"),
        b if b == PS_EXHAUSTED => Some("is exhausted"),
        b if b == PS_BEING_DRAGGED => Some("is being dragged"),
        b if b == PS_PICKED_UP => Some("has been picked up"),
        b if b == PS_HIT_ON_GROUND => Some("was hit while on the ground"),
        b if b == PS_SETUP_PREVENTED => Some("can not be set up"),
        b if b == PS_IN_THE_AIR => Some("is in the air"),
        _ => None,
    }
}

/// Java: `PlayerGender.getDative()` — not yet ported to the shared enum, translated
/// locally here. Verified against
/// `ffb-java/ffb-common/src/main/java/com/fumbbl/ffb/PlayerGender.java`: dative in enum
/// order MALE, FEMALE, NONBINARY, NEUTRAL = him, her, them, it.
///
/// Only consumer in `InjuryMessage.java` is the `thickSkullUsed` branch (`"'s Thick Skull
/// helps " + defender.getPlayerGender().getDative() + " to stay on the pitch."`), which is
/// itself gapped below (see the `// java: injuryModifierTotal / thickSkullUsed /
/// stuntyUsed` comment) because detecting Thick Skull/Stunty usage requires the
/// `InjuryModifier -> registered Skill` link that isn't retained in `ReportInjury`. Kept
/// here, `#[allow(dead_code)]`, so it's ready the moment that data becomes available.
#[allow(dead_code)]
fn dative(gender: ffb_model::enums::PlayerGender) -> &'static str {
    use ffb_model::enums::PlayerGender::*;
    match gender {
        Male => "him",
        Female => "her",
        Nonbinary => "them",
        Neutral => "it",
    }
}

/// Java: `SkipInjuryParts` is read off `report.getSkip()` directly as an enum; the Rust
/// `ReportInjury::skip` field only retains its `Display` name string, so this reverses
/// that formatting back into the enum. Not present on `SkipInjuryParts` itself since
/// nothing else in the codebase yet needs to parse it back from a string.
fn parse_skip(name: &str) -> SkipInjuryParts {
    match name {
        "ARMOUR" => SkipInjuryParts::Armour,
        "ARMOUR_AND_CAS" => SkipInjuryParts::ArmourAndCas,
        "ARMOUR_AND_INJURY" => SkipInjuryParts::ArmourAndInjury,
        "EVERYTHING_BUT_CAS" => SkipInjuryParts::EverythingButCas,
        "INJURY" => SkipInjuryParts::Injury,
        "CAS" => SkipInjuryParts::Cas,
        _ => SkipInjuryParts::None,
    }
}

impl ReportMessage for InjuryMessage {
    type Report = ReportInjury;

    fn report_id(&self) -> ReportId {
        ReportId::INJURY
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let defender = report.get_defender_id().and_then(|id| game.player(id));
        // java: `Player<?> attacker = game.getPlayerById(report.getAttackerId())` — only used by
        // the armour/injury modifier magnitude math below, which is gapped (see comments at its
        // call sites), so `attacker` itself has no remaining use on the Rust side.
        let _attacker = report.get_attacker_id().and_then(|id| game.player(id));
        let indent = status_report.get_indent();

        // java: report.getInjuryType().reportInjuryString(status, attacker, defender) — the
        // per-injury-type narrative (Chainsaw/Stab/Foul/etc. wording) lives entirely on the
        // Java `InjuryType` hierarchy, which isn't ported client-side; `ReportInjury` only
        // retains the type's plain name string here, so this composed sentence has no
        // faithful equivalent to render.

        // report armour roll
        let armor_roll = report.get_armor_roll();
        let skip = parse_skip(report.get_skip());
        if !armor_roll.is_empty() && !skip.is_armour() {
            status_report.println_indent_style(
                indent,
                TextStyle::ROLL,
                &format!("Armour Roll [ {} ][ {} ]", armor_roll[0], armor_roll[1]),
            );
            let rolled_total = armor_roll[0] + armor_roll[1];

            // java: armorModifierTotal / usingClaws / cancelingAvModifiers — computed from
            // `ArmorModifier.getModifier(attacker, defender)` and
            // `IRegistrationAwareModifier.getRegisteredTo()` per modifier instance. Only
            // modifier *names* survive in `report.get_armor_modifiers()` (no magnitude, no
            // registered-skill link), so the " + N Name = total" breakdown, the Claws usage
            // line, and the "ignores armour modifiers due to ..." line can't be reconstructed
            // without inventing numbers/skills that aren't in the data.
            status_report.println_indent(indent + 1, &format!("Rolled Total of {rolled_total}"));

            if let Some(defender) = defender {
                if report.is_armor_broken() {
                    status_report.print_indent(indent + 1, "The armour of ");
                    print_player(status_report, game, indent + 1, false, Some(defender));
                    status_report.println_indent(indent + 1, " has been broken.");
                } else {
                    print_player(status_report, game, indent + 1, false, Some(defender));
                    status_report.println_indent(
                        indent + 1,
                        &format!(" has been saved by {} armour.", defender.gender.genitive()),
                    );
                }
            }
        }

        // report injury roll
        if report.is_armor_broken() {
            let injury_roll = report.get_injury_roll();
            if !injury_roll.is_empty() && !skip.is_injury() {
                status_report.println_indent_style(
                    indent,
                    TextStyle::ROLL,
                    &format!("Injury Roll [ {} ][ {} ]", injury_roll[0], injury_roll[1]),
                );
                let rolled_total = injury_roll[0] + injury_roll[1];

                // java: injuryModifierTotal / thickSkullUsed / stuntyUsed — same gap as the
                // armour roll above: `InjuryModifier.getModifier(attacker, defender)` and
                // `isRegisteredToSkillWithProperty(convertKOToStunOn8 / isHurtMoreEasily)` need
                // the modifier object + registered skill, neither of which survive in
                // `report.get_injury_modifiers()` (plain name strings only).
                status_report.println_indent(indent + 1, &format!("Rolled Total of {rolled_total}"));
            }

            if !report.get_casualty_roll().is_empty() {
                if !skip.is_injury() {
                    print_player(status_report, game, indent + 1, false, defender);
                    status_report.println_indent(indent + 1, " suffers a casualty.");
                }
                if !skip.is_cas() {
                    if let Some(defender) = defender {
                        if defender.is_zapped() {
                            status_report.println_indent_style(
                                indent,
                                TextStyle::NONE,
                                &format!(
                                    "{} is badly hurt automatically because {} has been zapped.",
                                    defender.name,
                                    defender.gender.nominative()
                                ),
                            );
                        } else {
                            let casualty_roll = report.get_casualty_roll();
                            status_report.println_indent_style(
                                indent,
                                TextStyle::ROLL,
                                &format!("Casualty Roll [ {} ]", casualty_roll[0]),
                            );

                            // java: CasualtyModifier.reportString()/getModifier() total math —
                            // only modifier name strings are retained in
                            // `report.get_casualty_modifiers()`, so the "Rolled X + name = Y"
                            // breakdown can't be reconstructed without inventing magnitudes.

                            let mut factory = SeriousInjuryFactory::new();
                            factory.initialize(game);
                            let serious_injury = report.get_serious_injury().and_then(|n| factory.for_name(n));
                            let original_injury = report.get_original_injury().and_then(|n| factory.for_name(n));
                            report_injury(
                                status_report,
                                game,
                                Some(defender),
                                report.get_injury(),
                                serious_injury,
                                casualty_roll[1],
                                original_injury,
                            );
                        }
                    }
                }
            } else if report.get_injury().is_some() && !skip.is_injury() {
                report_injury(status_report, game, defender, report.get_injury(), None, 0, None);
            }
        }
    }
}

/// Java: `InjuryMessage.reportInjury(Player<?>, PlayerState, SeriousInjury, int, SeriousInjury)`.
#[allow(clippy::too_many_arguments)]
fn report_injury(
    status_report: &mut StatusReport,
    game: &Game,
    defender: Option<&Player>,
    injury: Option<PlayerState>,
    serious_injury: Option<AnySeriousInjury>,
    si_roll: i32,
    original_injury: Option<AnySeriousInjury>,
) {
    let indent = status_report.get_indent() + 1;
    print_player(status_report, game, indent, false, defender);
    if let Some(injury) = injury {
        if let Some(description) = player_state_description(injury) {
            status_report.println_indent(indent, &format!(" {description}."));
        }
    }
    if let Some(serious_injury) = serious_injury {
        if serious_injury.show_si_roll() {
            status_report.println_indent_style(
                status_report.get_indent(),
                TextStyle::ROLL,
                &format!("Lasting Injury Roll [ {si_roll} ]"),
            );
        }
        if let (Some(original_injury), Some(defender)) = (original_injury, defender) {
            status_report.println_indent_style(
                indent,
                TextStyle::EXPLANATION,
                &format!(
                    "{} would have {} but that stat cannot be reduced any further. The result counts as Miss Next Game instead.",
                    defender.name,
                    original_injury.get_description()
                ),
            );
        }
        print_player(status_report, game, indent, false, defender);
        status_report.println_indent(indent, &format!(" {}.", serious_injury.get_description()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerGender, PlayerType, Rules};
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

    fn make_player(id: &str, name: &str, gender: PlayerGender) -> Player {
        Player {
            id: id.into(),
            name: name.into(),
            gender,
            player_type: PlayerType::default(),
            ..Default::default()
        }
    }

    fn make_team(id: &str, players: Vec<Player>) -> Team {
        Team {
            id: id.into(),
            name: format!("Team {id}"),
            race: "Human".into(),
            roster_id: "human".into(),
            coach: "Coach".into(),
            rerolls: 0,
            apothecaries: 0,
            bribes: 0,
            master_chefs: 0,
            prayers_to_nuffle: 0,
            bloodweiser_kegs: 0,
            riotous_rookies: 0,
            cheerleaders: 0,
            assistant_coaches: 0,
            fan_factor: 0,
            dedicated_fans: 0,
            team_value: 0,
            treasury: 0,
            special_rules: vec![],
            players,
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn make_game() -> Game {
        let home = make_team("home", vec![make_player("a1", "Attacker", PlayerGender::Male)]);
        let away = make_team("away", vec![make_player("d1", "Defender", PlayerGender::Female)]);
        Game::new(home, away, Rules::Bb2025)
    }

    fn base_report(armor_broken: bool) -> ReportInjury {
        ReportInjury::new(
            Some("a1".into()),
            Some("d1".into()),
            "REGULAR".into(),
            armor_broken,
            vec![],
            vec![4, 5],
            vec![],
            vec![],
            vec![],
            None,
            vec![],
            None,
            None,
            None,
            None,
            vec![],
            "NONE".into(),
        )
    }

    #[test]
    fn armour_saved_prints_rolled_total_and_saved_line() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let report = base_report(false);
        InjuryMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t.contains("Armour Roll [ 4 ][ 5 ]")));
        assert!(texts.iter().any(|t| t.contains("Rolled Total of 9")));
        assert!(texts.iter().any(|t| t.contains("has been saved by her armour.")));
    }

    #[test]
    fn armour_broken_prints_broken_line_and_injury_roll() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let mut report = base_report(true);
        report.injury_roll = vec![2, 3];
        report.injury = Some(PlayerState::new(ffb_model::enums::PS_KNOCKED_OUT));
        InjuryMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t.contains("has been broken.")));
        assert!(texts.iter().any(|t| t.contains("Injury Roll [ 2 ][ 3 ]")));
        assert!(texts.iter().any(|t| t.contains("has been knocked out.")));
    }

    #[test]
    fn casualty_roll_reports_injury_and_serious_injury() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let mut report = base_report(true);
        report.injury_roll = vec![5, 6];
        report.casualty_roll = vec![7, 42];
        report.injury = Some(PlayerState::new(ffb_model::enums::PS_SERIOUS_INJURY));
        report.serious_injury = Some("SERIOUSLY_HURT".into());
        InjuryMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t.contains("suffers a casualty.")));
        assert!(texts.iter().any(|t| t.contains("Casualty Roll [ 7 ]")));
        assert!(texts.iter().any(|t| t.contains("has been seriously injured.")));
    }

    #[test]
    fn zapped_defender_reports_automatic_cas() {
        let mut status_report = StatusReport::new();
        let mut game = make_game();
        game.team_away.players[0].zapped = true;
        let mut report = base_report(true);
        report.casualty_roll = vec![7, 42];
        InjuryMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(texts.iter().any(|t| t.contains("is badly hurt automatically because she has been zapped.")));
    }

    #[test]
    fn skip_armour_suppresses_armour_roll_line() {
        let mut status_report = StatusReport::new();
        let game = make_game();
        let mut report = base_report(false);
        report.skip = "ARMOUR".into();
        InjuryMessage.render(&mut status_report, &game, &report);
        let texts: Vec<_> = status_report.rendered_runs.iter().filter_map(|r| r.text.clone()).collect();
        assert!(!texts.iter().any(|t| t.contains("Armour Roll")));
    }

    #[test]
    fn report_id_is_injury() {
        assert_eq!(InjuryMessage.report_id(), ReportId::INJURY);
    }
}
