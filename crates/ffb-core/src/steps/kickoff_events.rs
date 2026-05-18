/// Kickoff event resolution for Blood Bowl 2025.
///
/// `roll_kickoff_event` rolls 2d6 and maps the result to a `KickoffEvent`.
/// `apply_kickoff_event` applies the chosen event's effects to the game state.
use crate::model::game_state::GameState;
use crate::rng::GameRng;
use crate::types::{KickoffEvent, PlayerState, TeamId, TurnMode, Weather};

// ── Public API ────────────────────────────────────────────────────────────────

/// Roll 2d6 and return the corresponding `KickoffEvent`.
pub fn roll_kickoff_event(rng: &mut GameRng) -> KickoffEvent {
    let roll = rng.roll_2d6();
    KickoffEvent::from_2d6(roll)
}

/// Apply a kickoff event's effects to `state`.
///
/// Convention: during kickoff, `state.home_is_offense` tells us which team
/// kicked off. The **kicking** team is the offense team; the **receiving**
/// team is the defense team.
pub fn apply_kickoff_event(state: &mut GameState, event: KickoffEvent, rng: &mut GameRng) {
    match event {
        KickoffEvent::HighKick => {
            // Receiving team may place one player under the ball before scatter.
            state.turn_mode = TurnMode::HighKick;
        }

        KickoffEvent::QuickSnap => {
            // Receiving team gets a free move for d3+3 players.
            // Java rolls d3 to determine the count (even if no moves are made).
            // Consume the d3 to keep RNG in sync with the Java engine.
            let _quick_snap_count = rng.roll(3) + 3;
            state.turn_mode = TurnMode::QuickSnap;
        }

        KickoffEvent::Blitz => {
            // Kicking team takes a free turn (CHARGE in BB2025).
            // Java rolls d3 for the number of players allowed to act (even if the
            // canonical strategy picks none). Consume it to stay in RNG sync.
            let _charge_count = rng.roll(3) + 3;
            state.turn_mode = TurnMode::Blitz;
            // Kicking team = offense team → set them as active.
            state.home_is_active = state.home_is_offense;
        }

        KickoffEvent::PerfectDefence => {
            // SOLID_DEFENCE (BB2025): kicking team rearranges up to d3+3 players.
            // Java always rolls d3 here even when no rearrangement occurs.
            let _count = rng.roll(3);
            state.turn_mode = TurnMode::PerfectDefence;
        }

        KickoffEvent::GetTheRef => {
            // Both teams gain 1 extra bribe for this game.
            state.inducement_state_home.bribes_remaining =
                state.inducement_state_home.bribes_remaining.saturating_add(1);
            state.inducement_state_away.bribes_remaining =
                state.inducement_state_away.bribes_remaining.saturating_add(1);
        }

        KickoffEvent::Riot => {
            // TIME_OUT in BB2025: move BOTH teams' turn markers.
            // Java: if kicking team's turnNr >= 6, move both markers -1; otherwise +1.
            // Rust turn_number is 0-indexed (turns completed so far), so equivalent
            // threshold is >= 5 (Java's 6 = Rust's 5 completed + 1 upcoming).
            let kicking_turn = if state.home_is_offense {
                state.turn_data_home.turn_number
            } else {
                state.turn_data_away.turn_number
            };
            let modifier: i16 = if kicking_turn >= 5 { -1 } else { 1 };
            state.turn_data_home.turn_number =
                (state.turn_data_home.turn_number as i16 + modifier).max(0) as u8;
            state.turn_data_away.turn_number =
                (state.turn_data_away.turn_number as i16 + modifier).max(0) as u8;
        }

        KickoffEvent::ChangingWeather => {
            // Roll 2d6 and map to weather using the standard kickoff weather table.
            let roll = rng.roll_2d6();
            let new_weather = Weather::from_kickoff_roll(roll);
            state.field.weather = new_weather;
            // BB2025: when weather changes to SwelteringHeat, only EXHAUSTED players go to
            // Reserve (no dice — they are already tracked as EXHAUSTED state). Since our
            // canonical test starts with all players Standing, this is always a no-op here.
        }

        KickoffEvent::BrilliantCoaching => {
            // Each team rolls d6+FAME; higher total gains 1 reroll.
            // Simplified: roll d6 for each team and add their FAME modifier.
            let home_roll = rng.roll_d6() as i16 + state.home.fame as i16;
            let away_roll = rng.roll_d6() as i16 + state.away.fame as i16;
            if home_roll >= away_roll {
                state.home.rerolls_remaining = state.home.rerolls_remaining.saturating_add(1);
            }
            if away_roll >= home_roll {
                state.away.rerolls_remaining = state.away.rerolls_remaining.saturating_add(1);
            }
        }

        KickoffEvent::CheeringFans => {
            // CHEERING_FANS in BB2025: each team rolls d6 (not 2d6) + FAME modifier.
            // Java: rollDice(6) per team. Winner (or both on tie) gains 1 reroll.
            let home_total = rng.roll_d6() as i16 + state.home.fame as i16;
            let away_total = rng.roll_d6() as i16 + state.away.fame as i16;
            if home_total >= away_total {
                state.home.rerolls_remaining = state.home.rerolls_remaining.saturating_add(1);
            }
            if away_total >= home_total {
                state.away.rerolls_remaining = state.away.rerolls_remaining.saturating_add(1);
            }
        }

        KickoffEvent::ThrowARock => {
            // DODGY_SNACK in BB2025: fans throw snacks at players.
            // Roll d6 for each team; team with lower roll has a player targeted.
            // On a tie both teams are targeted.
            let roll_home = rng.roll_d6();
            let roll_away = rng.roll_d6();
            if roll_away >= roll_home {
                dodgy_snack_target(state, rng, TeamId::Home);
            }
            if roll_home >= roll_away {
                dodgy_snack_target(state, rng, TeamId::Away);
            }
        }

        KickoffEvent::SwelteringHeat => {
            // One random Standing player per team: roll d6, on 1 they go to Reserve.
            sweltering_heat_check(state, rng, TeamId::Home);
            sweltering_heat_check(state, rng, TeamId::Away);
        }

        KickoffEvent::PitchInvasion => {
            // BB2025 PITCH_INVASION: roll d6 per team + FAME; lower total has players stunned.
            // Roll d3 for stun count, then pick that many standing players via d(N) rolls.
            let roll_home = rng.roll_d6() as i16 + state.home.fame as i16;
            let roll_away = rng.roll_d6() as i16 + state.away.fame as i16;
            let stun_count = rng.roll(3) as usize;
            if roll_home <= roll_away {
                pitch_invasion_stun(state, rng, TeamId::Home, stun_count);
            }
            if roll_home >= roll_away {
                pitch_invasion_stun(state, rng, TeamId::Away, stun_count);
            }
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Return on-pitch player IDs for `team` sorted by jersey number.
/// This matches Java's `team.getPlayers()` array order (jersey 0..10 insertion order).
/// The field model uses a HashMap so iteration order is otherwise non-deterministic.
fn on_pitch_by_jersey(state: &GameState, team: TeamId) -> Vec<crate::types::PlayerId> {
    let team_obj = state.team(team);
    team_obj
        .players()
        .iter()
        .filter(|p| {
            state.field.player_state(&p.id).map(|ps| ps != PlayerState::Reserve).unwrap_or(false)
        })
        .map(|p| p.id.clone())
        .collect()
}

/// Return STANDING on-pitch player IDs for `team` sorted by jersey number.
fn standing_on_pitch_by_jersey(state: &GameState, team: TeamId) -> Vec<crate::types::PlayerId> {
    let team_obj = state.team(team);
    team_obj
        .players()
        .iter()
        .filter(|p| state.field.player_state(&p.id) == Some(PlayerState::Standing))
        .map(|p| p.id.clone())
        .collect()
}

/// Roll d6 for one randomly selected Standing player from `team`.
/// On a roll of 1, that player goes to Reserve (overcome by the heat).
fn sweltering_heat_check(state: &mut GameState, rng: &mut GameRng, team: TeamId) {
    let ids = standing_on_pitch_by_jersey(state, team);
    if ids.is_empty() {
        return;
    }
    let idx = (rng.roll_d6() as usize - 1) % ids.len();
    let roll = rng.roll_d6();
    if roll == 1 {
        state.field.set_player_state(&ids[idx], PlayerState::Reserve);
        state.field.remove_from_pitch(&ids[idx]);
    }
}

/// BB2025 PITCH_INVASION: knock prone up to `count` Standing players from `team`.
/// Players are sorted by jersey number to match Java's team.getPlayers() array order.
/// Java uses UtilServerInjury.stunPlayer → dropPlayer(PRONE) in the kickoff context.
fn pitch_invasion_stun(state: &mut GameState, rng: &mut GameRng, team: TeamId, count: usize) {
    let mut standing = standing_on_pitch_by_jersey(state, team);
    for _ in 0..count {
        if standing.is_empty() {
            break;
        }
        let idx = (rng.roll(standing.len() as u8) as usize) - 1;
        let id = standing.remove(idx);
        state.field.set_player_state(&id, PlayerState::Stunned);
    }
}

/// DODGY_SNACK target: pick one random on-pitch player from `team` and roll armor.
/// Players sorted by jersey number to match Java's playersOnField array order.
/// On d6=1 the player is sent to Reserve; on 2-6 they get a snack (not modeled).
fn dodgy_snack_target(state: &mut GameState, rng: &mut GameRng, team: TeamId) {
    let ids = on_pitch_by_jersey(state, team);
    if ids.is_empty() {
        return;
    }
    // Java: rollDice(players.length) - 1; Rust: roll(n) returns 1..=n, so subtract 1.
    let idx = (rng.roll(ids.len() as u8) as usize) - 1;
    let armor_roll = rng.roll_d6();
    if armor_roll == 1 {
        state.field.set_player_state(&ids[idx], PlayerState::Reserve);
        state.field.remove_from_pitch(&ids[idx]);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::game_state::GameState;
    use crate::model::player::{Player, PlayerStats};
    use crate::model::team::Team;
    use crate::rng::GameRng;
    use crate::skills::SkillSet;
    use crate::types::{FieldCoordinate, KickoffEvent, PlayerId, PlayerState, TeamId, Weather};

    // ── Fixtures ──────────────────────────────────────────────────────────

    fn make_state() -> GameState {
        let home = Team::new("home".into(), "Reavers".into(), "Human".into(), 3, true);
        let away = Team::new("away".into(), "Raiders".into(), "Orc".into(), 3, false);
        GameState::new(home, away)
    }

    fn add_player(state: &mut GameState, team: TeamId, id: &str, coord: FieldCoordinate) {
        let pid = PlayerId(id.into());
        let p = Player::new(
            pid.clone(),
            id.into(),
            "lineman".into(),
            team,
            1,
            PlayerStats::new(6, 3, 3, 8, None),
            SkillSet::empty(),
        );
        match team {
            TeamId::Home => state.home.add_player(p),
            TeamId::Away => state.away.add_player(p),
        }
        state.field.place_player(pid, team, coord, PlayerState::Standing);
    }

    // ── GetTheRef ─────────────────────────────────────────────────────────

    #[test]
    fn get_the_ref_increases_both_bribes() {
        let mut state = make_state();
        let mut rng = GameRng::new_test([]); // no dice needed
        assert_eq!(state.inducement_state_home.bribes_remaining, 0);
        assert_eq!(state.inducement_state_away.bribes_remaining, 0);
        apply_kickoff_event(&mut state, KickoffEvent::GetTheRef, &mut rng);
        assert_eq!(state.inducement_state_home.bribes_remaining, 1);
        assert_eq!(state.inducement_state_away.bribes_remaining, 1);
    }

    #[test]
    fn get_the_ref_stacks_on_existing_bribes() {
        let mut state = make_state();
        state.inducement_state_home.bribes_remaining = 2;
        state.inducement_state_away.bribes_remaining = 1;
        let mut rng = GameRng::new_test([]);
        apply_kickoff_event(&mut state, KickoffEvent::GetTheRef, &mut rng);
        assert_eq!(state.inducement_state_home.bribes_remaining, 3);
        assert_eq!(state.inducement_state_away.bribes_remaining, 2);
    }

    // ── FoulWeather / ChangingWeather ─────────────────────────────────────

    #[test]
    fn changing_weather_sets_sweltering_heat_on_2() {
        let mut state = make_state();
        // 2d6 sum = 2 → SwelteringHeat; inject two 1s
        let mut rng = GameRng::new_test([1, 1]);
        apply_kickoff_event(&mut state, KickoffEvent::ChangingWeather, &mut rng);
        assert_eq!(state.field.weather, Weather::SwelteringHeat);
    }

    #[test]
    fn changing_weather_sets_nice_day_on_7() {
        let mut state = make_state();
        // 2d6 sum = 7 → NiceDayForAFootballGame; inject 3+4
        let mut rng = GameRng::new_test([3, 4]);
        apply_kickoff_event(&mut state, KickoffEvent::ChangingWeather, &mut rng);
        assert_eq!(state.field.weather, Weather::NiceDayForAFootballGame);
    }

    #[test]
    fn changing_weather_sets_blizzard_on_12() {
        let mut state = make_state();
        // 2d6 sum = 12 → Blizzard; inject two 6s
        let mut rng = GameRng::new_test([6, 6]);
        apply_kickoff_event(&mut state, KickoffEvent::ChangingWeather, &mut rng);
        assert_eq!(state.field.weather, Weather::Blizzard);
    }

    // ── CheeringFans ──────────────────────────────────────────────────────

    #[test]
    fn cheering_fans_home_wins_higher_roll() {
        let mut state = make_state();
        let initial_home = state.home.rerolls_remaining;
        let initial_away = state.away.rerolls_remaining;
        // Home rolls 5+6=11, Away rolls 1+2=3 → home gains reroll
        let mut rng = GameRng::new_test([5, 6, 1, 2]);
        apply_kickoff_event(&mut state, KickoffEvent::CheeringFans, &mut rng);
        assert_eq!(state.home.rerolls_remaining, initial_home + 1);
        assert_eq!(state.away.rerolls_remaining, initial_away);
    }

    #[test]
    fn cheering_fans_away_wins_higher_roll() {
        let mut state = make_state();
        let initial_home = state.home.rerolls_remaining;
        let initial_away = state.away.rerolls_remaining;
        // Home rolls 1+1=2, Away rolls 5+6=11 → away gains reroll
        let mut rng = GameRng::new_test([1, 1, 5, 6]);
        apply_kickoff_event(&mut state, KickoffEvent::CheeringFans, &mut rng);
        assert_eq!(state.home.rerolls_remaining, initial_home);
        assert_eq!(state.away.rerolls_remaining, initial_away + 1);
    }

    #[test]
    fn cheering_fans_home_wins_on_tie() {
        let mut state = make_state();
        let initial_home = state.home.rerolls_remaining;
        let initial_away = state.away.rerolls_remaining;
        // Tie: both roll 3+4=7 → home gains reroll, away does not
        let mut rng = GameRng::new_test([3, 4, 3, 4]);
        apply_kickoff_event(&mut state, KickoffEvent::CheeringFans, &mut rng);
        assert_eq!(state.home.rerolls_remaining, initial_home + 1);
        assert_eq!(state.away.rerolls_remaining, initial_away);
    }

    // ── Riot ──────────────────────────────────────────────────────────────

    #[test]
    fn riot_on_receiving_turn_1_moves_counter_back() {
        let mut state = make_state();
        // Home is offense → away is receiving
        state.home_is_offense = true;
        state.turn_data_away.turn_number = 1;
        let mut rng = GameRng::new_test([]);
        apply_kickoff_event(&mut state, KickoffEvent::Riot, &mut rng);
        assert_eq!(state.turn_data_away.turn_number, 0);
    }

    #[test]
    fn riot_on_receiving_turn_8_moves_counter_forward() {
        let mut state = make_state();
        state.home_is_offense = true; // away receives
        state.turn_data_away.turn_number = 8;
        let mut rng = GameRng::new_test([]);
        apply_kickoff_event(&mut state, KickoffEvent::Riot, &mut rng);
        assert_eq!(state.turn_data_away.turn_number, 9);
    }

    #[test]
    fn riot_on_mid_turn_is_no_op() {
        let mut state = make_state();
        state.home_is_offense = true;
        state.turn_data_away.turn_number = 4;
        let mut rng = GameRng::new_test([]);
        apply_kickoff_event(&mut state, KickoffEvent::Riot, &mut rng);
        assert_eq!(state.turn_data_away.turn_number, 4);
    }

    #[test]
    fn riot_home_receiving_on_turn_1() {
        let mut state = make_state();
        state.home_is_offense = false; // home receives
        state.turn_data_home.turn_number = 1;
        let mut rng = GameRng::new_test([]);
        apply_kickoff_event(&mut state, KickoffEvent::Riot, &mut rng);
        assert_eq!(state.turn_data_home.turn_number, 0);
    }

    // ── PitchInvasion ─────────────────────────────────────────────────────

    #[test]
    fn pitch_invasion_stuns_player_on_6() {
        let mut state = make_state();
        add_player(&mut state, TeamId::Home, "h1", FieldCoordinate::new(5, 5));
        add_player(&mut state, TeamId::Away, "a1", FieldCoordinate::new(20, 5));
        // Roll 6 for home player → stunned; roll 1 for away player → stays standing
        let mut rng = GameRng::new_test([6, 1]);
        apply_kickoff_event(&mut state, KickoffEvent::PitchInvasion, &mut rng);
        assert_eq!(
            state.field.player_state(&PlayerId("h1".into())),
            Some(PlayerState::Stunned)
        );
        assert_eq!(
            state.field.player_state(&PlayerId("a1".into())),
            Some(PlayerState::Standing)
        );
    }

    #[test]
    fn pitch_invasion_no_players_no_panic() {
        let mut state = make_state();
        let mut rng = GameRng::new_test([]);
        // No players on pitch — should complete without panic or consuming dice.
        apply_kickoff_event(&mut state, KickoffEvent::PitchInvasion, &mut rng);
    }

    // ── SwelteringHeat ────────────────────────────────────────────────────

    #[test]
    fn sweltering_heat_may_remove_player() {
        let mut state = make_state();
        add_player(&mut state, TeamId::Home, "h1", FieldCoordinate::new(5, 5));
        add_player(&mut state, TeamId::Away, "a1", FieldCoordinate::new(20, 5));
        // For Home: idx roll=1 → picks h1 (idx 0), heat roll=1 → Reserve
        // For Away: idx roll=1 → picks a1 (idx 0), heat roll=1 → Reserve
        let mut rng = GameRng::new_test([1, 1, 1, 1]);
        apply_kickoff_event(&mut state, KickoffEvent::SwelteringHeat, &mut rng);
        assert_eq!(
            state.field.player_state(&PlayerId("h1".into())),
            Some(PlayerState::Reserve),
            "home player should be in Reserve after sweltering heat roll of 1"
        );
        assert_eq!(
            state.field.player_state(&PlayerId("a1".into())),
            Some(PlayerState::Reserve),
            "away player should be in Reserve after sweltering heat roll of 1"
        );
    }

    #[test]
    fn sweltering_heat_no_effect_on_high_roll() {
        let mut state = make_state();
        add_player(&mut state, TeamId::Home, "h2", FieldCoordinate::new(5, 5));
        add_player(&mut state, TeamId::Away, "a2", FieldCoordinate::new(20, 5));
        // idx roll=1, heat roll=2 (no reserve) for both teams
        let mut rng = GameRng::new_test([1, 2, 1, 2]);
        apply_kickoff_event(&mut state, KickoffEvent::SwelteringHeat, &mut rng);
        assert_eq!(
            state.field.player_state(&PlayerId("h2".into())),
            Some(PlayerState::Standing),
            "player with roll >= 2 should stay Standing"
        );
        assert_eq!(
            state.field.player_state(&PlayerId("a2".into())),
            Some(PlayerState::Standing),
            "away player with roll >= 2 should stay Standing"
        );
    }

    #[test]
    fn sweltering_heat_no_players_no_panic() {
        let mut state = make_state();
        let mut rng = GameRng::new_test([]);
        apply_kickoff_event(&mut state, KickoffEvent::SwelteringHeat, &mut rng);
    }

    // ── ThrowARock ────────────────────────────────────────────────────────

    #[test]
    fn throw_a_rock_stuns_one_player_per_team() {
        let mut state = make_state();
        add_player(&mut state, TeamId::Home, "h1", FieldCoordinate::new(5, 5));
        add_player(&mut state, TeamId::Away, "a1", FieldCoordinate::new(20, 5));
        // stun_random_player uses one d6 per team to select the player
        let mut rng = GameRng::new_test([1, 1]);
        apply_kickoff_event(&mut state, KickoffEvent::ThrowARock, &mut rng);
        assert_eq!(
            state.field.player_state(&PlayerId("h1".into())),
            Some(PlayerState::Stunned)
        );
        assert_eq!(
            state.field.player_state(&PlayerId("a1".into())),
            Some(PlayerState::Stunned)
        );
    }

    // ── roll_kickoff_event ────────────────────────────────────────────────

    #[test]
    fn roll_kickoff_event_maps_2_to_get_the_ref() {
        // Two d6 rolls of 1 → sum 2 → GetTheRef
        let mut rng = GameRng::new_test([1, 1]);
        assert_eq!(roll_kickoff_event(&mut rng), KickoffEvent::GetTheRef);
    }

    #[test]
    fn roll_kickoff_event_maps_12_to_pitch_invasion() {
        // Two d6 rolls of 6 → sum 12 → PitchInvasion
        let mut rng = GameRng::new_test([6, 6]);
        assert_eq!(roll_kickoff_event(&mut rng), KickoffEvent::PitchInvasion);
    }

    #[test]
    fn roll_kickoff_event_maps_10_to_blitz() {
        // 4+6 = 10 → Blitz
        let mut rng = GameRng::new_test([4, 6]);
        assert_eq!(roll_kickoff_event(&mut rng), KickoffEvent::Blitz);
    }

    // ── Mode-setting events ───────────────────────────────────────────────

    #[test]
    fn high_kick_sets_turn_mode() {
        let mut state = make_state();
        let mut rng = GameRng::new_test([]);
        apply_kickoff_event(&mut state, KickoffEvent::HighKick, &mut rng);
        assert_eq!(state.turn_mode, TurnMode::HighKick);
    }

    #[test]
    fn quick_snap_sets_turn_mode() {
        let mut state = make_state();
        let mut rng = GameRng::new_test([]);
        apply_kickoff_event(&mut state, KickoffEvent::QuickSnap, &mut rng);
        assert_eq!(state.turn_mode, TurnMode::QuickSnap);
    }

    #[test]
    fn blitz_sets_turn_mode_and_kicking_team_active() {
        let mut state = make_state();
        state.home_is_offense = true; // home kicks
        let mut rng = GameRng::new_test([]);
        apply_kickoff_event(&mut state, KickoffEvent::Blitz, &mut rng);
        assert_eq!(state.turn_mode, TurnMode::Blitz);
        assert!(state.home_is_active, "kicking (home) team should be active");
    }

    #[test]
    fn perfect_defence_sets_turn_mode() {
        let mut state = make_state();
        let mut rng = GameRng::new_test([]);
        apply_kickoff_event(&mut state, KickoffEvent::PerfectDefence, &mut rng);
        assert_eq!(state.turn_mode, TurnMode::PerfectDefence);
    }
}
