use crate::client::report::report_message_base::{print_player, ReportMessage};
use crate::client::status_report::StatusReport;
use crate::client::text_style::TextStyle;
use ffb_model::enums::{
    PlayerGender, PlayerState, PS_BADLY_HURT, PS_BANNED, PS_BEING_DRAGGED, PS_BLOCKED, PS_EXHAUSTED,
    PS_FALLING, PS_HIT_ON_GROUND, PS_IN_THE_AIR, PS_KNOCKED_OUT, PS_MISSING, PS_MOVING, PS_PICKED_UP,
    PS_PRONE, PS_RESERVE, PS_RIP, PS_SERIOUS_INJURY, PS_SETUP_PREVENTED, PS_STANDING, PS_STUNNED,
};
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::report::mixed::report_injury::ReportInjury;
use ffb_model::report::report_id::ReportId;
use ffb_model::report::skip_injury_parts::SkipInjuryParts;

/// 1:1 translation of `InjuryMessage.java`.
///
/// `ReportInjury.getArmorModifiers()`/`getInjuryModifiers()`/`getCasualtyModifiers()` in Java
/// return arrays of live `ArmorModifier`/`InjuryModifier`/`CasualtyModifier` objects with
/// context-dependent `getModifier(attacker, defender)`, `isFoulAssistModifier()`, and
/// `isRegisteredToSkillWithProperty(NamedProperties)` methods. The Rust `ReportInjury`
/// (`crates/ffb-model/src/report/mixed/report_injury.rs`) already reduces these to
/// `Vec<String>` — Debug-formatted dumps of `ffb_mechanics::modifiers::Modifier { name, value,
/// rules }` produced by `build_report_injury` in `crates/ffb-engine/src/mechanic/state_mechanic.rs`
/// (outside this translation's allowed scope: `crates/ffb-client/src/client/report/bb2020/`).
/// `parse_modifier_debug` below recovers `(name, value)` from that Debug string so the roll-total
/// arithmetic can still be reproduced faithfully; the skill-registration flags
/// (`isRegisteredToSkillWithProperty`, `isFoulAssistModifier`, `getRegisteredTo`) have no
/// surviving data at all, so the derived Claws/cancelling-armour-modifiers/Stunty/Thick-Skull
/// call-outs cannot be reconstructed — see the `// java: gap` comments at each omitted branch.
pub struct InjuryMessage;

/// Java: `PlayerState.getDescription()` (`ffb-common/.../PlayerState.java`) — not yet ported to
/// the Rust `PlayerState` (`crates/ffb-model/src/enums/player.rs`).
fn player_state_description(state: PlayerState) -> &'static str {
    match state.base() {
        PS_STANDING => "is standing",
        PS_MOVING => "is moving",
        PS_PRONE => "is prone",
        PS_STUNNED => "has been stunned",
        PS_KNOCKED_OUT => "has been knocked out",
        PS_BADLY_HURT => "has been badly hurt",
        PS_SERIOUS_INJURY => "has been seriously injured",
        PS_RIP => "has been killed",
        PS_RESERVE => "is in reserve",
        PS_MISSING => "is missing the game",
        PS_FALLING => "is about to fall down",
        PS_BLOCKED => "is being blocked",
        PS_BANNED => "is banned from the game",
        PS_EXHAUSTED => "is exhausted",
        PS_BEING_DRAGGED => "is being dragged",
        PS_PICKED_UP => "has been picked up",
        PS_HIT_ON_GROUND => "was hit while on the ground",
        PS_SETUP_PREVENTED => "can not be set up",
        PS_IN_THE_AIR => "is in the air",
        _ => "is unknown",
    }
}

/// Java: `PlayerGender.getDative()` — not yet exposed on the Rust `PlayerGender` enum
/// (`crates/ffb-model/src/enums/player.rs` only has `.nominative()`/`.genitive()`).
fn player_gender_dative(gender: PlayerGender) -> &'static str {
    match gender {
        PlayerGender::Male => "him",
        PlayerGender::Female => "her",
        PlayerGender::Nonbinary => "them",
        PlayerGender::Neutral => "it",
    }
}

/// Java: `bb2020.SeriousInjury.getDescription()`. `ReportInjury.serious_injury`/
/// `serious_injury_decay`/`original_injury` are already reduced to a Debug-formatted
/// `SeriousInjuryKind` variant name (e.g. `"SeriouslyHurt"`) by `build_report_injury`
/// (`crates/ffb-engine/src/mechanic/state_mechanic.rs`, outside this file's scope), so the
/// real `SeriousInjury` object (with its edition-specific description table) is unreachable
/// from here. This local table reproduces `crates/ffb-model/src/bb2020/serious_injury.rs`'s
/// `get_description()` for the BB2020 variant names this BB2020-only message can see.
fn serious_injury_description(kind_debug_name: &str) -> &'static str {
    match kind_debug_name {
        "SeriouslyHurt" => "is seriously hurt (Miss next game)",
        "SeriousInjuryNi" => "is seriously injured (Niggling Injury)",
        "HeadInjuryAv" => "suffered a head injury (-1 AV)",
        "SmashedKneeMa" => "suffered a smashed knee (-1 MA)",
        "BrokenArmPa" => "suffered a broken arm (-1 PA)",
        "NeckInjuryAg" => "suffered a neck injury (-1 AG)",
        "DislocatedShoulderSt" => "suffered a dislocated shoulder (-1 ST)",
        "Dead" => "is dead",
        _ => "",
    }
}

/// Java: `bb2020.SeriousInjury.showSiRoll()` — see `serious_injury_description` for why this
/// operates on the Debug-formatted `SeriousInjuryKind` name rather than the real object.
fn serious_injury_show_si_roll(kind_debug_name: &str) -> bool {
    matches!(
        kind_debug_name,
        "HeadInjuryAv" | "SmashedKneeMa" | "BrokenArmPa" | "NeckInjuryAg" | "DislocatedShoulderSt"
    )
}

/// Java: `report.getSkip()` returns a `SkipInjuryParts` enum constant directly; the Rust
/// `ReportInjury.skip` field stores its `Display` string (see
/// `crates/ffb-model/src/report/skip_injury_parts.rs`) instead, so this parses it back to reuse
/// the existing `is_armour`/`is_injury`/`is_cas` methods faithfully.
fn parse_skip(skip: &str) -> SkipInjuryParts {
    match skip {
        "ARMOUR" => SkipInjuryParts::Armour,
        "ARMOUR_AND_CAS" => SkipInjuryParts::ArmourAndCas,
        "ARMOUR_AND_INJURY" => SkipInjuryParts::ArmourAndInjury,
        "EVERYTHING_BUT_CAS" => SkipInjuryParts::EverythingButCas,
        "INJURY" => SkipInjuryParts::Injury,
        "CAS" => SkipInjuryParts::Cas,
        _ => SkipInjuryParts::None,
    }
}

/// Recovers `(name, value)` from a Debug-formatted `ffb_mechanics::modifiers::Modifier` string
/// (`Modifier { name: "Mighty Blow +1", value: 1, rules: Bb2020 }`). See the module-level doc
/// comment for why this parsing step exists.
fn parse_modifier_debug(debug_str: &str) -> Option<(String, i32)> {
    let name_start = debug_str.find("name: \"")? + "name: \"".len();
    let name_end = name_start + debug_str[name_start..].find('"')?;
    let name = debug_str[name_start..name_end].to_string();
    let value_start = debug_str.find("value: ")? + "value: ".len();
    let value_rest = &debug_str[value_start..];
    let value_end = value_rest.find(',')?;
    let value: i32 = value_rest[..value_end].trim().parse().ok()?;
    Some((name, value))
}

/// Java: `InjuryType.reportInjuryString(StringBuilder, Player, Player)`. The base class default
/// is a no-op (returns without appending); the Rust `ReportInjury.injury_type` field is already
/// reduced to a bare type-name `String` (`crates/ffb-model/src/report/mixed/report_injury.rs`),
/// with no ported per-subtype overrides (`ffb-common/.../injury/InjuryType.java` subclasses),
/// so this always matches the default no-op behaviour.
fn injury_type_report_string(_injury_type: &str, _attacker: Option<&Player>, _defender: Option<&Player>) -> String {
    String::new()
}

impl ReportMessage for InjuryMessage {
    type Report = ReportInjury;

    fn report_id(&self) -> ReportId {
        ReportId::INJURY
    }

    fn render(&self, status_report: &mut StatusReport, game: &Game, report: &Self::Report) {
        let indent = status_report.get_indent();
        let defender = report.get_defender_id().and_then(|id| game.player(id));
        let attacker = report.get_attacker_id().and_then(|id| game.player(id));
        let skip = parse_skip(report.get_skip());

        // report injury type
        let injury_type_status = injury_type_report_string(report.get_injury_type(), attacker, defender);
        if !injury_type_status.is_empty() {
            status_report.println_indent(indent + 1, &injury_type_status);
        }

        // report armour roll
        let armor_roll = report.get_armor_roll();
        if armor_roll.len() >= 2 && !skip.is_armour() {
            let mut status = format!("Armour Roll [ {} ][ {} ]", armor_roll[0], armor_roll[1]);
            status_report.println_indent_style(indent, TextStyle::ROLL, &status);
            let rolled_total = armor_roll[0] + armor_roll[1];
            status = format!("Rolled Total of {rolled_total}");
            let mut armor_modifier_total = 0;
            // java: `usingClaws`/`cancelingAvModifiers` (derived from
            // `isRegisteredToSkillWithProperty(NamedProperties.reducesArmourToFixedValue /
            // ignoresArmourModifiersFrom*)`) cannot be recovered from the Debug-formatted
            // `Modifier` strings — no registration data survives — so the "uses Claws to
            // reduce opponent's armour to 8+" and "ignores armour modifiers due to ..."
            // call-outs are never rendered here.
            for raw_modifier in report.get_armor_modifiers() {
                if let Some((name, value)) = parse_modifier_debug(raw_modifier) {
                    if value != 0 {
                        armor_modifier_total += value;
                        if value > 0 {
                            status.push_str(" + ");
                        } else {
                            status.push_str(" - ");
                        }
                        // java: `!armorModifier.isFoulAssistModifier()` gate on printing the
                        // numeric value — that flag has no surviving data, so the value is
                        // always printed.
                        status.push_str(&format!("{} ", value.abs()));
                        status.push_str(&name);
                    }
                }
            }
            if armor_modifier_total != 0 {
                status.push_str(&format!(" = {}", rolled_total + armor_modifier_total));
            }
            status_report.println_indent(indent + 1, &status);
            if defender.is_some() {
                if report.is_armor_broken() {
                    status_report.print_indent(indent + 1, "The armour of ");
                    print_player(status_report, game, indent + 1, false, defender);
                    status_report.println_indent(indent + 1, " has been broken.");
                } else {
                    print_player(status_report, game, indent + 1, false, defender);
                    let genitive = defender.map(|d| d.gender.genitive()).unwrap_or("");
                    status_report.println_indent(indent + 1, &format!(" has been saved by {genitive} armour."));
                }
            }
        }

        // report injury roll
        if report.is_armor_broken() {
            let injury_roll = report.get_injury_roll();
            if injury_roll.len() >= 2 && !skip.is_injury() {
                let mut status = format!("Injury Roll [ {} ][ {} ]", injury_roll[0], injury_roll[1]);
                status_report.println_indent_style(indent, TextStyle::ROLL, &status);
                let rolled_total = injury_roll[0] + injury_roll[1];
                status = format!("Rolled Total of {rolled_total}");
                let mut injury_modifier_total = 0;
                // java: `thickSkullUsed`/`stuntyUsed` (derived from
                // `isRegisteredToSkillWithProperty(NamedProperties.convertKOToStunOn8 /
                // isHurtMoreEasily)` when `modifierValue == 0`) cannot be recovered from the
                // Debug-formatted `Modifier` strings, so the "is Stunty and more easily hurt"
                // and "'s Thick Skull helps ... to stay on the pitch" call-outs are never
                // rendered here.
                for raw_modifier in report.get_injury_modifiers() {
                    if let Some((name, value)) = parse_modifier_debug(raw_modifier) {
                        injury_modifier_total += value;
                        if value > 0 {
                            status.push_str(&format!(" + {value} {name}"));
                        } else if value < 0 {
                            status.push_str(&format!(" {value} {name}"));
                        }
                    }
                }
                if injury_modifier_total != 0 {
                    status.push_str(&format!(" = {}", rolled_total + injury_modifier_total));
                }
                status_report.println_indent(indent + 1, &status);
            }

            let casualty_roll = report.get_casualty_roll();
            if casualty_roll.len() >= 2 && !skip.is_cas() {
                print_player(status_report, game, indent + 1, false, defender);
                status_report.println_indent(indent + 1, " suffers a casualty.");
                if defender.is_some_and(|d| game.is_zapped_player(&d.id)) {
                    // java: `defender instanceof ZappedPlayer` — Rust has no subclass
                    // hierarchy for `Player`; `Game.is_zapped_player` (backed by
                    // `Game.zapped_players: Vec<ZappedPlayer>`) is the closest faithful
                    // equivalent to "this defender was killed by a zap effect".
                    let defender = defender.unwrap();
                    let status = format!(
                        "{} is badly hurt automatically because {} has been zapped.",
                        defender.name,
                        defender.gender.nominative()
                    );
                    status_report.println_indent_style(indent, TextStyle::NONE, &status);
                } else {
                    let mut status = format!("Casualty Roll [ {} ]", casualty_roll[0]);
                    status_report.println_indent_style(indent, TextStyle::ROLL, &status);
                    let casualty_modifiers = report.get_casualty_modifiers();
                    if !casualty_modifiers.is_empty() {
                        let mut modifiers_total = 0;
                        let mut report_strings: Vec<String> = Vec::new();
                        for raw_modifier in casualty_modifiers {
                            if let Some((name, value)) = parse_modifier_debug(raw_modifier) {
                                report_strings.push(format!("{value} {name}"));
                                modifiers_total += value;
                            }
                        }
                        report_strings.sort();
                        status = format!("Rolled {}", casualty_roll[0]);
                        for report_string in &report_strings {
                            status.push_str(" + ");
                            status.push_str(report_string);
                        }
                        status.push_str(&format!(" = {}", casualty_roll[0] + modifiers_total));
                        status_report.println_indent_style(indent + 1, TextStyle::NONE, &status);
                    }
                    report_injury(
                        status_report,
                        game,
                        indent,
                        defender,
                        report.get_injury(),
                        report.get_serious_injury(),
                        casualty_roll[1],
                        report.get_original_injury(),
                    );
                }
            } else if report.get_injury().is_some() && !skip.is_injury() {
                report_injury(status_report, game, indent, defender, report.get_injury(), None, 0, None);
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn report_injury(
    status_report: &mut StatusReport,
    game: &Game,
    indent: i32,
    defender: Option<&Player>,
    injury: Option<PlayerState>,
    serious_injury: Option<&str>,
    si_roll: i32,
    original_injury: Option<&str>,
) {
    let Some(injury) = injury else { return };
    print_player(status_report, game, indent + 1, false, defender);
    status_report.println_indent(indent + 1, &format!(" {}.", player_state_description(injury)));
    if let Some(serious_injury) = serious_injury {
        if serious_injury_show_si_roll(serious_injury) {
            status_report.println_indent_style(indent, TextStyle::ROLL, &format!("Lasting Injury Roll [ {si_roll} ]"));
        }
        if let Some(original_injury) = original_injury {
            if let Some(defender) = defender {
                let status = format!(
                    "{} would have {} but that stat cannot be reduced any further. So a different injury has been chosen randomly.",
                    defender.name,
                    serious_injury_description(original_injury)
                );
                status_report.println_indent_style(indent + 1, TextStyle::EXPLANATION, &status);
            }
        }
        print_player(status_report, game, indent + 1, false, defender);
        status_report.println_indent(indent + 1, &format!(" {}.", serious_injury_description(serious_injury)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::team::Team;
    use ffb_model::model::zapped_player::ZappedPlayer;

    fn make_team(id: &str) -> Team {
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
            players: vec![],
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn make_game() -> Game {
        let mut home = make_team("home");
        home.players.push(Player {
            id: "def1".into(),
            name: "Defender".into(),
            gender: PlayerGender::Male,
            ..Player::default()
        });
        let mut away = make_team("away");
        away.players.push(Player {
            id: "att1".into(),
            name: "Attacker".into(),
            gender: PlayerGender::Male,
            ..Player::default()
        });
        Game::new(home, away, Rules::Bb2020)
    }

    #[allow(clippy::too_many_arguments)]
    fn make_report(
        armor_broken: bool,
        armor_roll: Vec<i32>,
        injury_roll: Vec<i32>,
        casualty_roll: Vec<i32>,
        casualty_modifiers: Vec<String>,
        serious_injury: Option<String>,
        injury: Option<PlayerState>,
        original_injury: Option<String>,
    ) -> ReportInjury {
        ReportInjury::new(
            Some("att1".to_string()),
            Some("def1".to_string()),
            "REGULAR".to_string(),
            armor_broken,
            vec![],
            armor_roll,
            vec![],
            injury_roll,
            casualty_roll,
            serious_injury,
            vec![],
            None,
            original_injury,
            injury,
            None,
            casualty_modifiers,
            "NONE".to_string(),
        )
    }

    #[test]
    fn report_id_matches() {
        assert_eq!(InjuryMessage.report_id(), ReportId::INJURY);
    }

    #[test]
    fn armour_saved_when_not_broken() {
        let mut sr = StatusReport::new();
        let game = make_game();
        let report = make_report(false, vec![3, 4], vec![], vec![], vec![], None, None, None);
        InjuryMessage.render(&mut sr, &game, &report);
        assert!(sr.rendered_runs.iter().any(|r| r.text.as_deref() == Some("Armour Roll [ 3 ][ 4 ]")));
        assert!(sr.rendered_runs.iter().any(|r| r.text.as_deref().is_some_and(|t| t.contains("has been saved by his armour."))));
    }

    #[test]
    fn armour_broken_injury_roll_no_casualty() {
        let mut sr = StatusReport::new();
        let game = make_game();
        let report = make_report(true, vec![5, 6], vec![2, 3], vec![], vec![], None, None, None);
        InjuryMessage.render(&mut sr, &game, &report);
        assert!(sr.rendered_runs.iter().any(|r| r.text.as_deref().is_some_and(|t| t.contains("has been broken."))));
        assert!(sr.rendered_runs.iter().any(|r| r.text.as_deref() == Some("Injury Roll [ 2 ][ 3 ]")));
        assert!(!sr.rendered_runs.iter().any(|r| r.text.as_deref().is_some_and(|t| t.contains("suffers a casualty"))));
    }

    #[test]
    fn armour_broken_casualty_with_modifiers_and_serious_injury() {
        let mut sr = StatusReport::new();
        let game = make_game();
        let report = make_report(
            true,
            vec![5, 6],
            vec![4, 5],
            vec![7, 3],
            vec!["Modifier { name: \"Mighty Blow +1\", value: 1, rules: Bb2020 }".to_string()],
            Some("HeadInjuryAv".to_string()),
            Some(PlayerState::new(PS_BADLY_HURT)),
            None,
        );
        InjuryMessage.render(&mut sr, &game, &report);
        assert!(sr.rendered_runs.iter().any(|r| r.text.as_deref().is_some_and(|t| t.contains("suffers a casualty."))));
        assert!(sr.rendered_runs.iter().any(|r| r.text.as_deref() == Some("Casualty Roll [ 7 ]")));
        assert!(sr.rendered_runs.iter().any(|r| r.text.as_deref().is_some_and(|t| t.contains("Rolled 7 + 1 Mighty Blow +1 = 8"))));
        assert!(sr.rendered_runs.iter().any(|r| r.text.as_deref().is_some_and(|t| t.contains("has been badly hurt."))));
        assert!(sr.rendered_runs.iter().any(|r| r.text.as_deref() == Some("Lasting Injury Roll [ 3 ]")));
        assert!(sr.rendered_runs.iter().any(|r| r.text.as_deref().is_some_and(|t| t.contains("suffered a head injury (-1 AV)."))));
    }

    #[test]
    fn zapped_defender_reports_automatic_casualty() {
        let mut sr = StatusReport::new();
        let mut game = make_game();
        let defender = game.team_home.player("def1").unwrap().clone();
        game.zapped_players.push(ZappedPlayer::new(defender, ffb_model::model::zapped_position::ZappedPosition::default()));
        let report = make_report(true, vec![5, 6], vec![4, 5], vec![7, 3], vec![], None, None, None);
        InjuryMessage.render(&mut sr, &game, &report);
        assert!(sr.rendered_runs.iter().any(|r| r.text.as_deref().is_some_and(|t| t.contains("is badly hurt automatically because he has been zapped."))));
    }

    #[test]
    fn player_state_description_matches_java_table() {
        assert_eq!(player_state_description(PlayerState::new(PS_STANDING)), "is standing");
        assert_eq!(player_state_description(PlayerState::new(PS_BADLY_HURT)), "has been badly hurt");
        assert_eq!(player_state_description(PlayerState::new(PS_RIP)), "has been killed");
    }

    #[test]
    fn parse_modifier_debug_recovers_name_and_value() {
        let parsed = parse_modifier_debug("Modifier { name: \"Dirty Player +1\", value: 1, rules: Bb2020 }");
        assert_eq!(parsed, Some(("Dirty Player +1".to_string(), 1)));
    }
}
