use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::types::{FieldCoordinate, PlayerId, PlayerState, TeamId, Weather, PITCH_SQUARES};

// ── Ball state ────────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub struct BallState {
    pub in_play: bool,
    pub moving: bool,
    pub coord: Option<FieldCoordinate>,
}

// ── Tackle zone map ───────────────────────────────────────────────────────────

/// Per-square count of opposing tackle zones for each team.
/// `home[idx]` = number of Away players adjacent to square `idx`.
/// `away[idx]` = number of Home players adjacent to square `idx`.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct TacklezoneMap {
    /// Zones threatening home players (created by away players)
    pub home: Vec<u8>,
    /// Zones threatening away players (created by home players)
    pub away: Vec<u8>,
}

impl Default for TacklezoneMap {
    fn default() -> Self {
        Self {
            home: vec![0; PITCH_SQUARES],
            away: vec![0; PITCH_SQUARES],
        }
    }
}

impl TacklezoneMap {
    /// Number of opposing tackle zones on `coord` from the perspective of `team`.
    pub fn zones_on(&self, coord: FieldCoordinate, team: TeamId) -> u8 {
        if !coord.is_valid() {
            return 0;
        }
        match team {
            TeamId::Home => self.home[coord.index()],
            TeamId::Away => self.away[coord.index()],
        }
    }

    pub fn in_tackle_zone(&self, coord: FieldCoordinate, team: TeamId) -> bool {
        self.zones_on(coord, team) > 0
    }

    fn add_zones_for(&mut self, coord: FieldCoordinate, team: TeamId) {
        // A player at `coord` belonging to `team` threatens all adjacent squares
        // for the opposing team.
        let target = match team {
            TeamId::Home => &mut self.away,
            TeamId::Away => &mut self.home,
        };
        for n in coord.neighbors() {
            target[n.index()] = target[n.index()].saturating_add(1);
        }
    }

    fn remove_zones_for(&mut self, coord: FieldCoordinate, team: TeamId) {
        let target = match team {
            TeamId::Home => &mut self.away,
            TeamId::Away => &mut self.home,
        };
        for n in coord.neighbors() {
            target[n.index()] = target[n.index()].saturating_sub(1);
        }
    }
}

// ── FieldModel ────────────────────────────────────────────────────────────────

/// The game board: player placements, states, and ball position.
/// The flat `placement` array maps coordinate index → Option<PlayerId>.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct FieldModel {
    /// Coordinate index → player at that square (len = PITCH_SQUARES = 442)
    placement: Vec<Option<PlayerId>>,
    /// Player id → current state
    states: HashMap<PlayerId, PlayerState>,
    /// Player id → current coordinate (for on-pitch players)
    coords: HashMap<PlayerId, FieldCoordinate>,
    /// Player id → team (needed for TZ updates)
    teams: HashMap<PlayerId, TeamId>,
    pub ball: BallState,
    pub weather: Weather,
    pub tackle_zones: TacklezoneMap,
}

impl Default for FieldModel {
    fn default() -> Self {
        Self {
            placement: vec![None; PITCH_SQUARES],
            states: HashMap::new(),
            coords: HashMap::new(),
            teams: HashMap::new(),
            ball: BallState::default(),
            weather: Weather::default(),
            tackle_zones: TacklezoneMap::default(),
        }
    }
}

impl FieldModel {
    pub fn new() -> Self {
        Self::default()
    }

    // ── Placement ─────────────────────────────────────────────────────────

    pub fn place_player(&mut self, id: PlayerId, team: TeamId, coord: FieldCoordinate, state: PlayerState) {
        debug_assert!(coord.is_valid());
        debug_assert!(self.placement[coord.index()].is_none(), "square already occupied");
        self.placement[coord.index()] = Some(id.clone());
        self.states.insert(id.clone(), state);
        self.coords.insert(id.clone(), coord);
        self.teams.insert(id, team);
        self.tackle_zones.add_zones_for(coord, team);
    }

    pub fn remove_player(&mut self, id: &PlayerId) {
        if let Some(coord) = self.coords.remove(id) {
            self.placement[coord.index()] = None;
            let team = self.teams.remove(id).expect("team for placed player");
            self.tackle_zones.remove_zones_for(coord, team);
        }
        self.states.remove(id);
    }

    /// Remove player from pitch coordinates/placement but keep their state entry.
    /// Used for crowd-pushed players: they leave the pitch but their Ko/Injured
    /// state remains queryable until they return at the next kickoff.
    pub fn remove_from_pitch(&mut self, id: &PlayerId) {
        if let Some(coord) = self.coords.remove(id) {
            self.placement[coord.index()] = None;
            let team = self.teams.remove(id).expect("team for placed player");
            self.tackle_zones.remove_zones_for(coord, team);
        }
        // state deliberately preserved
    }

    pub fn move_player(&mut self, id: &PlayerId, to: FieldCoordinate) {
        let from = *self.coords.get(id).expect("player not on pitch");
        debug_assert!(to.is_valid());
        debug_assert!(self.placement[to.index()].is_none(), "destination occupied");

        let team = *self.teams.get(id).expect("team for player");

        // Update TZ map
        self.tackle_zones.remove_zones_for(from, team);
        self.tackle_zones.add_zones_for(to, team);

        // Update placement
        self.placement[from.index()] = None;
        self.placement[to.index()] = Some(id.clone());
        *self.coords.get_mut(id).unwrap() = to;
    }

    // ── Queries ───────────────────────────────────────────────────────────

    pub fn player_at(&self, coord: FieldCoordinate) -> Option<&PlayerId> {
        if coord.is_valid() {
            self.placement[coord.index()].as_ref()
        } else {
            None
        }
    }

    pub fn player_state(&self, id: &PlayerId) -> Option<PlayerState> {
        self.states.get(id).copied()
    }

    pub fn set_player_state(&mut self, id: &PlayerId, state: PlayerState) {
        if let Some(s) = self.states.get_mut(id) {
            *s = state;
        }
    }

    pub fn player_coord(&self, id: &PlayerId) -> Option<FieldCoordinate> {
        self.coords.get(id).copied()
    }

    pub fn player_team(&self, id: &PlayerId) -> Option<TeamId> {
        self.teams.get(id).copied()
    }

    pub fn is_occupied(&self, coord: FieldCoordinate) -> bool {
        coord.is_valid() && self.placement[coord.index()].is_some()
    }

    /// All players currently on the pitch.
    pub fn on_pitch_players(&self) -> impl Iterator<Item = (&PlayerId, FieldCoordinate, PlayerState)> {
        self.coords.iter().map(|(id, &coord)| {
            let state = self.states[id];
            (id, coord, state)
        })
    }

    /// Players on the pitch for a given team.
    pub fn team_players_on_pitch(&self, team: TeamId) -> impl Iterator<Item = (&PlayerId, FieldCoordinate, PlayerState)> {
        self.on_pitch_players()
            .filter(move |(id, _, _)| self.teams.get(*id) == Some(&team))
    }

    pub fn tackle_zones_on(&self, coord: FieldCoordinate, team: TeamId) -> u8 {
        self.tackle_zones.zones_on(coord, team)
    }

    pub fn in_tackle_zone(&self, coord: FieldCoordinate, team: TeamId) -> bool {
        self.tackle_zones.in_tackle_zone(coord, team)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{FieldCoordinate, TeamId, PlayerId, PlayerState};

    fn pid(s: &str) -> PlayerId {
        PlayerId(s.into())
    }

    #[test]
    fn place_and_query() {
        let mut f = FieldModel::new();
        let coord = FieldCoordinate::new(5, 5);
        f.place_player(pid("p1"), TeamId::Home, coord, PlayerState::Standing);
        assert_eq!(f.player_at(coord), Some(&pid("p1")));
        assert_eq!(f.player_coord(&pid("p1")), Some(coord));
        assert_eq!(f.player_state(&pid("p1")), Some(PlayerState::Standing));
    }

    #[test]
    fn remove_player() {
        let mut f = FieldModel::new();
        let coord = FieldCoordinate::new(5, 5);
        f.place_player(pid("p1"), TeamId::Home, coord, PlayerState::Standing);
        f.remove_player(&pid("p1"));
        assert!(f.player_at(coord).is_none());
        assert!(f.player_coord(&pid("p1")).is_none());
    }

    #[test]
    fn move_player() {
        let mut f = FieldModel::new();
        let from = FieldCoordinate::new(5, 5);
        let to = FieldCoordinate::new(6, 5);
        f.place_player(pid("p1"), TeamId::Home, from, PlayerState::Standing);
        f.move_player(&pid("p1"), to);
        assert!(f.player_at(from).is_none());
        assert_eq!(f.player_at(to), Some(&pid("p1")));
        assert_eq!(f.player_coord(&pid("p1")), Some(to));
    }

    #[test]
    fn tackle_zone_incremented_on_place() {
        let mut f = FieldModel::new();
        // Home player at (5,5) → threatens away players adjacent to (5,5)
        let home_coord = FieldCoordinate::new(5, 5);
        f.place_player(pid("h1"), TeamId::Home, home_coord, PlayerState::Standing);
        // Away player at (6,5) is adjacent → should have 1 TZ from home
        let away_coord = FieldCoordinate::new(6, 5);
        assert_eq!(f.tackle_zones_on(away_coord, TeamId::Away), 1);
        // Home square itself: no TZ from home player
        assert_eq!(f.tackle_zones_on(home_coord, TeamId::Away), 0);
    }

    #[test]
    fn tackle_zone_decremented_on_remove() {
        let mut f = FieldModel::new();
        let home_coord = FieldCoordinate::new(5, 5);
        f.place_player(pid("h1"), TeamId::Home, home_coord, PlayerState::Standing);
        f.remove_player(&pid("h1"));
        let away_coord = FieldCoordinate::new(6, 5);
        assert_eq!(f.tackle_zones_on(away_coord, TeamId::Away), 0);
    }

    #[test]
    fn tackle_zone_updated_on_move() {
        let mut f = FieldModel::new();
        let from = FieldCoordinate::new(5, 5);
        let to = FieldCoordinate::new(10, 10);
        f.place_player(pid("h1"), TeamId::Home, from, PlayerState::Standing);

        // TZ present near 'from'
        assert_eq!(f.tackle_zones_on(FieldCoordinate::new(6, 5), TeamId::Away), 1);
        // No TZ near 'to' yet
        assert_eq!(f.tackle_zones_on(FieldCoordinate::new(11, 10), TeamId::Away), 0);

        f.move_player(&pid("h1"), to);

        // Old TZ gone
        assert_eq!(f.tackle_zones_on(FieldCoordinate::new(6, 5), TeamId::Away), 0);
        // New TZ present
        assert_eq!(f.tackle_zones_on(FieldCoordinate::new(11, 10), TeamId::Away), 1);
    }

    /// Property: after N random place/move/remove operations, TZ map equals
    /// one rebuilt from scratch.
    #[test]
    fn tackle_zone_consistency_after_random_ops() {
        use rand::{Rng, SeedableRng};
        let mut rng = rand::rngs::SmallRng::seed_from_u64(42);
        let mut f = FieldModel::new();
        let mut next_id = 0u32;
        let mut on_pitch: Vec<(PlayerId, TeamId)> = Vec::new();

        for _ in 0..500 {
            let op = rng.gen_range(0..3);
            match op {
                0 if on_pitch.len() < 20 => {
                    // Place
                    let id = PlayerId(format!("p{next_id}"));
                    next_id += 1;
                    let team = if rng.gen_bool(0.5) { TeamId::Home } else { TeamId::Away };
                    // Find an empty square
                    for _ in 0..10 {
                        let x = rng.gen_range(0..crate::types::PITCH_WIDTH);
                        let y = rng.gen_range(0..crate::types::PITCH_HEIGHT);
                        let coord = FieldCoordinate::new(x, y);
                        if !f.is_occupied(coord) {
                            f.place_player(id.clone(), team, coord, PlayerState::Standing);
                            on_pitch.push((id, team));
                            break;
                        }
                    }
                }
                1 if !on_pitch.is_empty() => {
                    // Move
                    let idx = rng.gen_range(0..on_pitch.len());
                    let (ref id, _team) = on_pitch[idx];
                    for _ in 0..10 {
                        let x = rng.gen_range(0..crate::types::PITCH_WIDTH);
                        let y = rng.gen_range(0..crate::types::PITCH_HEIGHT);
                        let coord = FieldCoordinate::new(x, y);
                        if !f.is_occupied(coord) {
                            f.move_player(id, coord);
                            break;
                        }
                    }
                }
                2 if !on_pitch.is_empty() => {
                    // Remove
                    let idx = rng.gen_range(0..on_pitch.len());
                    let (id, _) = on_pitch.swap_remove(idx);
                    f.remove_player(&id);
                }
                _ => {}
            }
        }

        // Rebuild TZ from scratch and compare
        let mut expected_home = vec![0u8; PITCH_SQUARES];
        let mut expected_away = vec![0u8; PITCH_SQUARES];
        for (id, coord, _state) in f.on_pitch_players() {
            let team = f.player_team(id).unwrap();
            for n in coord.neighbors() {
                match team {
                    TeamId::Home => expected_away[n.index()] += 1,
                    TeamId::Away => expected_home[n.index()] += 1,
                }
            }
        }
        assert_eq!(f.tackle_zones.home, expected_home, "home TZ mismatch");
        assert_eq!(f.tackle_zones.away, expected_away, "away TZ mismatch");
    }
}
