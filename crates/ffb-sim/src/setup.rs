/// Player placement and kickoff setup helpers.
use ffb_core::{FieldCoordinate, PlayerState, TeamId};
use ffb_core::model::game_state::GameState;
use ffb_core::model::player::{Player, PlayerStats};
use ffb_core::model::team::Team;
use ffb_core::skills::{SkillId, SkillSet};
use ffb_core::types::PlayerId;

/// Place all eligible (non-KO, non-Injured) players in a default formation
/// ready for kickoff.
///
/// - Home players: x ∈ 1..13, y ∈ 4..14
/// - Away players: x ∈ 13..25, y ∈ 4..14
/// - At most 11 players per team on the field.
pub fn place_players_for_kickoff(state: &mut GameState) {
    // Remove all currently placed players first
    let all_ids: Vec<_> = state.field
        .on_pitch_players()
        .map(|(id, _, _)| id.clone())
        .collect();
    for id in all_ids {
        state.field.remove_player(&id);
    }

    place_team_for_kickoff(state, TeamId::Home);
    place_team_for_kickoff(state, TeamId::Away);
}

fn place_team_for_kickoff(state: &mut GameState, team: TeamId) {
    // Collect eligible players sorted by jersey number (matches Java's jersey-sorted ordering).
    let mut eligible: Vec<_> = state.team(team)
        .players()
        .iter()
        .filter(|p| !matches!(
            state.field.player_state(&p.id),
            Some(PlayerState::Ko) | Some(PlayerState::Injured)
        ))
        .collect();
    eligible.sort_by_key(|p| p.jersey_number);
    let eligible_ids: Vec<_> = eligible.iter().map(|p| p.id.clone()).take(11).collect();

    // Canonical placement matching Java's ParityRunner.placeReserves():
    // LOS squares first (3 needed), then overflow squares.
    // Home LOS/overflow; Away squares are the mirror (x → 25-x, same y).
    let los_home: &[(u8, u8)] = &[
        (12,7),(12,6),(12,8),(12,5),(12,9),(12,4),(12,10),
    ];
    let overflow_home: &[(u8, u8)] = &[
        (5,5),(5,7),(5,9),(6,6),(6,8),(4,6),(4,8),(3,6),(3,8),(2,5),(2,9),(1,7),
    ];

    let coord_for = |raw: (u8, u8)| -> FieldCoordinate {
        match team {
            TeamId::Home => FieldCoordinate::new(raw.0, raw.1),
            // Mirror x: (25 - x)
            TeamId::Away => FieldCoordinate::new(25 - raw.0, raw.1),
        }
    };

    let n = eligible_ids.len();
    let los_needed = if n >= 3 { 3usize } else { n };
    let mut placed = 0usize;

    // Place on LOS
    for &sq in los_home {
        if placed >= los_needed { break; }
        let coord = coord_for(sq);
        if !state.field.is_occupied(coord) {
            state.field.place_player(eligible_ids[placed].clone(), team, coord, PlayerState::Standing);
            placed += 1;
        }
    }

    // Place remainder in overflow
    for &sq in overflow_home {
        if placed >= n { break; }
        let coord = coord_for(sq);
        if !state.field.is_occupied(coord) {
            state.field.place_player(eligible_ids[placed].clone(), team, coord, PlayerState::Standing);
            placed += 1;
        }
    }
}

// ── Roster loading ────────────────────────────────────────────────────────────

/// Load a team from a simple JSON roster format.
///
/// Format:
/// ```json
/// {
///   "name": "Reikland Reavers",
///   "race": "Human",
///   "rerolls": 3,
///   "apothecary": true,
///   "players": [
///     {
///       "id": "h0",
///       "name": "Griff",
///       "position": "blitzer",
///       "ma": 9, "st": 3, "ag": 4, "av": 8, "pa": null,
///       "skills": ["Block", "Dodge"]
///     }
///   ]
/// }
/// ```
pub fn load_team_from_json(json: &str, team_id: TeamId) -> Result<Team, String> {
    let value: serde_json::Value = serde_json::from_str(json)
        .map_err(|e| format!("JSON parse error: {e}"))?;

    let name = value["name"].as_str().ok_or("missing 'name'")?.to_string();
    let race = value["race"].as_str().ok_or("missing 'race'")?.to_string();
    let rerolls = value["rerolls"].as_u64().ok_or("missing 'rerolls'")? as u8;
    let apothecary = value["apothecary"].as_bool().unwrap_or(false);
    let team_id_str = match team_id {
        TeamId::Home => "home",
        TeamId::Away => "away",
    };

    let mut team = Team::new(team_id_str.to_string(), name, race, rerolls, apothecary);

    let players = value["players"].as_array().ok_or("missing 'players'")?;
    for (jersey_number, p) in players.iter().enumerate() {
        let id = p["id"].as_str().ok_or("player missing 'id'")?.to_string();
        let pname = p["name"].as_str().ok_or("player missing 'name'")?.to_string();
        let position = p["position"].as_str().ok_or("player missing 'position'")?.to_string();
        let ma = p["ma"].as_u64().ok_or("player missing 'ma'")? as u8;
        let st = p["st"].as_u64().ok_or("player missing 'st'")? as u8;
        let ag = p["ag"].as_u64().ok_or("player missing 'ag'")? as u8;
        let av = p["av"].as_u64().ok_or("player missing 'av'")? as u8;
        let pa = if p["pa"].is_null() {
            None
        } else {
            Some(p["pa"].as_u64().ok_or("player 'pa' must be integer or null")? as u8)
        };

        let mut skills = SkillSet::empty();
        if let Some(skill_arr) = p["skills"].as_array() {
            for s in skill_arr {
                let skill_name = s.as_str().ok_or("skill must be a string")?;
                let skill_id = parse_skill(skill_name)?;
                skills.add(skill_id);
            }
        }

        let player = Player::new(
            PlayerId(id),
            pname,
            position,
            team_id,
            (jersey_number + 1) as u8,
            PlayerStats::new(ma, st, ag, av, pa),
            skills,
        );
        team.add_player(player);
    }

    Ok(team)
}

/// Parse a skill name string into a `SkillId`.
fn parse_skill(name: &str) -> Result<SkillId, String> {
    match name {
        "Block" => Ok(SkillId::Block),
        "Catch" => Ok(SkillId::Catch),
        "Dauntless" => Ok(SkillId::Dauntless),
        "DisturbingPresence" => Ok(SkillId::DisturbingPresence),
        "DivingCatch" => Ok(SkillId::DivingCatch),
        "DumpOff" => Ok(SkillId::DumpOff),
        "ExtraArms" => Ok(SkillId::ExtraArms),
        "Fend" => Ok(SkillId::Fend),
        "FoulAppearance" => Ok(SkillId::FoulAppearance),
        "HailMaryPass" => Ok(SkillId::HailMaryPass),
        "Horns" => Ok(SkillId::Horns),
        "JumpUp" => Ok(SkillId::JumpUp),
        "Pass" => Ok(SkillId::Pass),
        "Sprint" => Ok(SkillId::Sprint),
        "StandFirm" => Ok(SkillId::StandFirm),
        "StripBall" => Ok(SkillId::StripBall),
        "SureHands" => Ok(SkillId::SureHands),
        "Tackle" => Ok(SkillId::Tackle),
        "Tentacles" => Ok(SkillId::Tentacles),
        "ThickSkull" => Ok(SkillId::ThickSkull),
        "TwoHeads" => Ok(SkillId::TwoHeads),
        "Wrestle" => Ok(SkillId::Wrestle),
        "BoneHead" => Ok(SkillId::BoneHead),
        "Dodge" => Ok(SkillId::Dodge),
        "Guard" => Ok(SkillId::Guard),
        "Juggernaut" => Ok(SkillId::Juggernaut),
        "Kick" => Ok(SkillId::Kick),
        "Leap" => Ok(SkillId::Leap),
        "Loner" => Ok(SkillId::Loner),
        "MightyBlow" => Ok(SkillId::MightyBlow),
        "Pro" => Ok(SkillId::Pro),
        "Regeneration" => Ok(SkillId::Regeneration),
        "SureFeet" => Ok(SkillId::SureFeet),
        "WildAnimal" => Ok(SkillId::WildAnimal),
        "Frenzy" => Ok(SkillId::Frenzy),
        "Stunty" => Ok(SkillId::Stunty),
        "TakeRoot" => Ok(SkillId::TakeRoot),
        "NervesOfSteel" => Ok(SkillId::NervesOfSteel),
        "OnTheBall" => Ok(SkillId::OnTheBall),
        "SafePairOfHands" => Ok(SkillId::SafePairOfHands),
        "Claws" => Ok(SkillId::Claws),
        "PrehensileTail" => Ok(SkillId::PrehensileTail),
        "DirtyPlayer" => Ok(SkillId::DirtyPlayer),
        "SideStep" => Ok(SkillId::SideStep),
        "Shadowing" => Ok(SkillId::Shadowing),
        "DivingTackle" => Ok(SkillId::DivingTackle),
        "BreakTackle" => Ok(SkillId::BreakTackle),
        _ => Err(format!("unknown skill: '{name}'")),
    }
}

/// Place the ball at the center square for the kicking team's side.
/// Default: x=13, y=8 (center of the pitch, boundary between halves).
pub fn default_kickoff_ball_placement(state: &mut GameState) {
    let coord = FieldCoordinate::new(13, 8);
    state.field.ball.coord = Some(coord);
    state.field.ball.in_play = true;
    state.field.ball.moving = false;
}

/// Scatter the ball from `kick_from` after kickoff (d8 direction, d6 distance).
/// Returns `(final_coord, is_touchback)`.
/// If the ball lands out of bounds (or in the kicking team's half), it is a
/// touchback — the receiving team places the ball at any player's square.
/// For automated play, the ball is placed at the nearest receiving team player.
pub fn perform_kickoff_scatter(
    state: &mut GameState,
    kick_from: ffb_core::types::FieldCoordinate,
    rng: &mut ffb_core::rng::GameRng,
) -> (ffb_core::types::FieldCoordinate, bool) {
    use ffb_core::pathfinding::apply_scatter;
    use ffb_core::types::FieldCoordinate;

    let direction = rng.roll_scatter_direction();
    let distance = rng.roll_scatter_distance();
    let landed = apply_scatter(kick_from, direction, distance);

    // Determine if ball is in receiving team's half.
    // Rust pitch: x=0-25. Home half: x=0-12, away half: x=13-25.
    // (Java: home kicks from (13,8), away kicks from (13,8).transform()=(12,8))
    let receiving_is_home = !state.home_is_offense;
    let receiving_start_x: u8 = if receiving_is_home { 0 } else { 13 };
    let receiving_end_x: u8 = if receiving_is_home { 12 } else { 25 };

    // Java HALF_HOME/HALF_AWAY: x=[rsx,rex], y=[0,14]. x=0 is home end zone (in HALF_HOME).
    let in_receiving_half = landed.x >= receiving_start_x && landed.x <= receiving_end_x && landed.y <= 14;

    let receiving_team = if receiving_is_home { ffb_core::types::TeamId::Home } else { ffb_core::types::TeamId::Away };

    if !in_receiving_half {
        // Touchback: find nearest receiving team player on pitch, give them ball
        let fallback = find_touchback_recipient(state, receiving_team, kick_from);
        state.field.ball.coord = Some(fallback);
        state.field.ball.in_play = true;
        (fallback, true)
    } else {
        state.field.ball.coord = Some(landed);
        state.field.ball.in_play = true;
        (landed, false)
    }
}

/// BB2025 post-kickoff catch/bounce sequence matching Java's CATCH_SCATTER_THROW_IN step.
///
/// Java has two CATCH_SCATTER_THROW_IN steps in the kickoff sequence, but the second is always
/// a no-op: the CATCH_KICKOFF parameter is consumed by the first step, leaving the second with
/// a null mode that causes it to exit immediately.
///
/// The first step handles everything in a single loop:
///   - If player at ball: try catch (d6). If success: done.
///   - Whether player present or not (and on catch fail): bounce (d8).
///   - If bounce stays in half and hits a player: loop back to try catch.
///   - If bounce out of half: touchback. If no player at new pos: ball settles.
///
/// Only call this when the initial kickoff scatter landed normally (not touchback).
/// When `scatter_was_touchback` is true, this function is a no-op (matches Java's
/// KICKOFF_ANIMATION not broadcasting CATCH_KICKOFF mode on touchback).
pub fn kickoff_bounce_if_needed(
    state: &mut GameState,
    kick_from: ffb_core::types::FieldCoordinate,
    scatter_was_touchback: bool,
    rng: &mut ffb_core::rng::GameRng,
) {
    if scatter_was_touchback {
        return;
    }

    let receiving_is_home = !state.home_is_offense;
    let receiving_start_x: u8 = if receiving_is_home { 0 } else { 13 };
    let receiving_end_x: u8 = if receiving_is_home { 12 } else { 25 };
    let receiving_team = if receiving_is_home {
        ffb_core::types::TeamId::Home
    } else {
        ffb_core::types::TeamId::Away
    };
    let kicking_team = if receiving_is_home {
        ffb_core::types::TeamId::Away
    } else {
        ffb_core::types::TeamId::Home
    };

    let ball_coord = match state.field.ball.coord {
        Some(c) => c,
        None => return,
    };
    // Java HALF_HOME/HALF_AWAY bounds: x=[rsx,rex], y=[0,14]. Include end zone (x=0 for home).
    let in_java_half = |c: ffb_core::types::FieldCoordinate| -> bool {
        c.x >= receiving_start_x && c.x <= receiving_end_x && c.y <= 14
    };
    if !in_java_half(ball_coord) {
        return;
    }

    // Single unified loop matching Java's StepCatchScatterThrowIn behavior:
    // - First catch attempt (at kickoff scatter landing): CATCH_KICKOFF mode — no scatter +1.
    // - On fail (or no player): bounce (d8). After the first bounce, subsequent catches use
    //   CATCH_SCATTER mode which adds +1 to the minimum roll ("Inaccurate Pass or Scatter").
    // - If new position has a player: loop back to catch attempt.
    // - If new position is empty (in bounds): ball settles, done.
    // - If out of bounds: touchback, done.
    let mut current = ball_coord;
    let mut scatter_modifier: u8 = 0; // +1 after first bounce (CATCH_SCATTER in Java)
    loop {
        // If receiving-team active player is here, attempt catch
        if let Some(pid) = state.field.player_at(current).cloned() {
            if state.field.player_team(&pid) == Some(receiving_team)
                && state.field.player_state(&pid).map(|s| s.is_active()).unwrap_or(false)
            {
                let ag = player_effective_ag(state, receiving_team, &pid);
                let opp_tz = state.field.tackle_zones_on(current, kicking_team);
                let min_roll = (ag + opp_tz + scatter_modifier).max(2);
                let roll = rng.roll(6);
                if catch_succeeds(roll, min_roll) {
                    state.field.ball.coord = Some(current);
                    return;
                }
            }
            // Player present but catch failed (or wrong team) → fall through to bounce
        }
        // No player at current pos, or catch failed → bounce (d8)
        // After first bounce, Java switches to CATCH_SCATTER mode (+1 scatter modifier)
        scatter_modifier = 1;
        let dir = rng.roll_scatter_direction();
        match bounce_in_half(current, dir, receiving_start_x, receiving_end_x) {
            None => {
                // Out of receiving half → touchback
                let tb = find_touchback_recipient(state, receiving_team, kick_from);
                state.field.ball.coord = Some(tb);
                return;
            }
            Some(new_pos) => {
                state.field.ball.coord = Some(new_pos);
                current = new_pos;
                if state.field.player_at(current).is_none() {
                    return; // No player at new pos → ball settles
                }
                // Player at new pos → loop to attempt catch
            }
        }
    }
}

/// Scatter ball one square from `from` in `direction`.
/// Returns `Some(new_pos)` if the new position is within the receiving half (Java bounds: y<=14).
/// Returns `None` if out of bounds (touchback needed).
fn bounce_in_half(
    from: ffb_core::types::FieldCoordinate,
    direction: u8,
    receiving_start_x: u8,
    receiving_end_x: u8,
) -> Option<ffb_core::types::FieldCoordinate> {
    let (dx, dy) = ffb_core::pathfinding::scatter_delta(direction);
    let raw_x = from.x as i16 + dx as i16;
    let raw_y = from.y as i16 + dy as i16;
    // Java HALF_HOME/HALF_AWAY: x=[rsx,rex], y=[0,14] (FIELD_HEIGHT=15, max y=14)
    if raw_x >= receiving_start_x as i16
        && raw_x <= receiving_end_x as i16
        && raw_y >= 0
        && raw_y <= 14
    {
        Some(ffb_core::types::FieldCoordinate::new(raw_x as u8, raw_y as u8))
    } else {
        None
    }
}

fn player_effective_ag(
    state: &GameState,
    team: ffb_core::types::TeamId,
    pid: &ffb_core::types::PlayerId,
) -> u8 {
    state.team(team)
        .player_by_id(pid)
        .map(|p| p.effective_ag())
        .unwrap_or(3)
}

/// After a kickoff with Changing Weather → Nice Day, scatter the ball up to 3 times
/// within the receiving half (matching Java's StepApplyKickoffResult.handleWeatherChange).
///
/// Returns `true` if the ball ended up out of bounds (touchback), `false` otherwise.
/// Only does anything when the current weather is NiceDayForAFootballGame AND
/// `scatter_was_touchback` is false.
pub fn apply_nice_weather_scatter(
    state: &mut GameState,
    scatter_was_touchback: bool,
    rng: &mut ffb_core::rng::GameRng,
) -> bool {
    use ffb_core::types::Weather;
    if scatter_was_touchback || state.field.weather != Weather::NiceDayForAFootballGame {
        return false;
    }

    let receiving_is_home = !state.home_is_offense;
    let receiving_start_x: u8 = if receiving_is_home { 0 } else { 13 };
    let receiving_end_x: u8 = if receiving_is_home { 12 } else { 25 };

    let mut current = match state.field.ball.coord {
        Some(c) => c,
        None => return false,
    };

    for _ in 0..3 {
        let dir = rng.roll_scatter_direction();
        match bounce_in_half(current, dir, receiving_start_x, receiving_end_x) {
            None => {
                // Out of receiving half → touchback; Java's StepTouchback picks nearest
                // receiving-team player to kick_from (same as perform_kickoff_scatter's logic).
                let receiving_team = if receiving_is_home {
                    ffb_core::types::TeamId::Home
                } else {
                    ffb_core::types::TeamId::Away
                };
                let kick_from = ffb_core::types::FieldCoordinate::new(13, 8);
                let tb = find_touchback_recipient(state, receiving_team, kick_from);
                state.field.ball.coord = Some(tb);
                return true;
            }
            Some(new_pos) => {
                current = new_pos;
                state.field.ball.coord = Some(new_pos);
            }
        }
    }
    false
}

fn catch_succeeds(roll: u8, minimum: u8) -> bool {
    roll == 6 || (roll != 1 && roll >= minimum)
}

fn find_touchback_recipient(
    state: &GameState,
    team: ffb_core::types::TeamId,
    near: ffb_core::types::FieldCoordinate,
) -> ffb_core::types::FieldCoordinate {
    use ffb_core::types::FieldCoordinate;
    // Find the standing player on the receiving team closest to `near`
    let mut best: Option<FieldCoordinate> = None;
    let mut best_dist = u32::MAX;
    for player in state.team(team).players() {
        if state.field.player_state(&player.id).map(|s| s.is_active()).unwrap_or(false) {
            if let Some(coord) = state.field.player_coord(&player.id) {
                let dx = (coord.x as i32 - near.x as i32).unsigned_abs();
                let dy = (coord.y as i32 - near.y as i32).unsigned_abs();
                let dist = dx * dx + dy * dy;
                if dist < best_dist {
                    best_dist = dist;
                    best = Some(coord);
                }
            }
        }
    }
    best.unwrap_or(FieldCoordinate::new(13, 8))
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_core::model::game_state::GameState;
    use ffb_core::model::player::{Player, PlayerStats};
    use ffb_core::model::team::Team;
    use ffb_core::skills::SkillSet;
    use ffb_core::types::{PlayerId, TeamId};

    fn make_full_game_state() -> GameState {
        let mut home = Team::new("h".into(), "Home".into(), "Human".into(), 3, true);
        for i in 0..11 {
            home.add_player(Player::new(
                PlayerId(format!("h{i}")),
                format!("HPlayer{i}"),
                "lineman".into(),
                TeamId::Home,
                i as u8 + 1,
                PlayerStats::new(6, 3, 4, 8, None),
                SkillSet::empty(),
            ));
        }

        let mut away = Team::new("a".into(), "Away".into(), "Orc".into(), 3, false);
        for i in 0..11 {
            away.add_player(Player::new(
                PlayerId(format!("a{i}")),
                format!("APlayer{i}"),
                "lineman".into(),
                TeamId::Away,
                i as u8 + 1,
                PlayerStats::new(5, 4, 3, 9, None),
                SkillSet::empty(),
            ));
        }

        GameState::new(home, away)
    }

    #[test]
    fn place_players_puts_players_on_field() {
        let mut state = make_full_game_state();
        place_players_for_kickoff(&mut state);

        let home_count = state.field
            .team_players_on_pitch(TeamId::Home)
            .count();
        let away_count = state.field
            .team_players_on_pitch(TeamId::Away)
            .count();
        assert_eq!(home_count, 11);
        assert_eq!(away_count, 11);
    }

    #[test]
    fn home_players_in_correct_zone() {
        let mut state = make_full_game_state();
        place_players_for_kickoff(&mut state);

        for (_, coord, _) in state.field.team_players_on_pitch(TeamId::Home) {
            assert!(coord.x >= 1 && coord.x < 13, "home player x={} out of zone", coord.x);
            assert!(coord.y >= 4 && coord.y < 14, "home player y={} out of zone", coord.y);
        }
    }

    #[test]
    fn away_players_in_correct_zone() {
        let mut state = make_full_game_state();
        place_players_for_kickoff(&mut state);

        for (_, coord, _) in state.field.team_players_on_pitch(TeamId::Away) {
            assert!(coord.x >= 13 && coord.x < 25, "away player x={} out of zone", coord.x);
            assert!(coord.y >= 4 && coord.y < 14, "away player y={} out of zone", coord.y);
        }
    }

    #[test]
    fn ball_placed_at_center() {
        let mut state = make_full_game_state();
        default_kickoff_ball_placement(&mut state);
        assert_eq!(state.field.ball.coord, Some(FieldCoordinate::new(13, 8)));
        assert!(state.field.ball.in_play);
    }

    const HUMAN_ROSTER_JSON: &str = r#"{
        "name": "Reikland Reavers",
        "race": "Human",
        "rerolls": 3,
        "apothecary": true,
        "players": [
            {
                "id": "h0",
                "name": "Griff Oberwald",
                "position": "blitzer",
                "ma": 9, "st": 3, "ag": 4, "av": 8, "pa": null,
                "skills": ["Block", "Dodge", "Sprint", "SureFeet"]
            },
            {
                "id": "h1",
                "name": "Lineman",
                "position": "lineman",
                "ma": 6, "st": 3, "ag": 4, "av": 8, "pa": null,
                "skills": []
            },
            {
                "id": "h2",
                "name": "Thrower",
                "position": "thrower",
                "ma": 6, "st": 3, "ag": 4, "av": 8, "pa": 3,
                "skills": ["Pass", "SureHands"]
            }
        ]
    }"#;

    #[test]
    fn load_human_roster_stats() {
        let team = load_team_from_json(HUMAN_ROSTER_JSON, TeamId::Home)
            .expect("roster should parse");
        assert_eq!(team.name, "Reikland Reavers");
        assert_eq!(team.race, "Human");
        assert_eq!(team.rerolls_total, 3);
        assert!(team.apothecary_available);
        assert_eq!(team.players().len(), 3);
    }

    #[test]
    fn load_human_roster_griff_skills() {
        use ffb_core::skills::SkillId;
        let team = load_team_from_json(HUMAN_ROSTER_JSON, TeamId::Home)
            .expect("roster should parse");
        let griff = team.player_by_id(&PlayerId("h0".into())).expect("griff");
        assert_eq!(griff.base_stats.ma, 9);
        assert_eq!(griff.base_stats.st, 3);
        assert_eq!(griff.base_stats.ag, 4);
        assert_eq!(griff.base_stats.av, 8);
        assert!(griff.base_stats.pa.is_none());
        assert!(griff.has_skill(SkillId::Block));
        assert!(griff.has_skill(SkillId::Dodge));
        assert!(griff.has_skill(SkillId::Sprint));
        assert!(griff.has_skill(SkillId::SureFeet));
    }

    #[test]
    fn load_human_roster_thrower_pa() {
        let team = load_team_from_json(HUMAN_ROSTER_JSON, TeamId::Home)
            .expect("roster should parse");
        let thrower = team.player_by_id(&PlayerId("h2".into())).expect("thrower");
        assert_eq!(thrower.base_stats.pa, Some(3));
    }

    #[test]
    fn load_team_invalid_json_returns_error() {
        let result = load_team_from_json("not json", TeamId::Home);
        assert!(result.is_err());
    }

    #[test]
    fn load_team_unknown_skill_returns_error() {
        let json = r#"{
            "name": "X",
            "race": "Human",
            "rerolls": 2,
            "players": [
                {"id": "p1", "name": "P", "position": "lineman",
                 "ma": 6, "st": 3, "ag": 4, "av": 8, "pa": null,
                 "skills": ["FakeSkill"]}
            ]
        }"#;
        let result = load_team_from_json(json, TeamId::Away);
        assert!(result.is_err());
    }
}
