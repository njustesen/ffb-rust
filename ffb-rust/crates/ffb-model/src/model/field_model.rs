use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use crate::enums::{CardEffect, Weather};
use crate::inducement::card::Card;
use crate::types::{FieldCoordinate, MoveSquare, PushbackSquare, RangeRuler};
use crate::enums::PlayerState;
use crate::model::player::PlayerId;
use crate::model::target_selection_state::TargetSelectionState;
use crate::model::dice_decoration::DiceDecoration;
use crate::marking::player_marker::PlayerMarker;
use crate::marking::field_marker::FieldMarker;
use crate::model::blood_spot::BloodSpot;
use crate::model::track_number::TrackNumber;

/// The game board: player positions, ball position, and transient UI state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldModel {
    pub weather: Weather,

    pub ball_coordinate: Option<FieldCoordinate>,
    pub ball_moving: bool,
    pub ball_in_play: bool,

    pub bomb_coordinate: Option<FieldCoordinate>,
    pub bomb_moving: bool,

    /// Java: FieldModel.fBallOutOfBounds — true when the ball lands out of bounds.
    pub out_of_bounds: bool,

    /// Maps player id → field coordinate. Absent = not on pitch.
    pub player_coordinates: HashMap<PlayerId, FieldCoordinate>,
    /// Arrival order per player: the value of `arrival_counter` when the player last CHANGED
    /// square. Java keeps `fPlayerIdByCoordinate` as an ordered list and `getPlayer(coordinate)`
    /// returns the LAST player to arrive; Rust's `player_at` scanned a HashMap, so when two
    /// players share a square (Java allows it — a shadower stepping onto a standing player's
    /// square) the answer changed from process to process.
    #[serde(skip)]
    pub coordinate_arrival: HashMap<PlayerId, u64>,
    #[serde(skip)]
    pub arrival_counter: u64,
    /// Maps player id → current PlayerState.
    pub player_states: HashMap<PlayerId, PlayerState>,

    pub range_ruler: Option<RangeRuler>,
    /// Java: FieldModel.fMoveSquares — keyed by coordinate for O(1) lookup via get_move_square.
    pub move_squares: HashMap<FieldCoordinate, MoveSquare>,
    pub pushback_squares: Vec<PushbackSquare>,

    /// Stadium trap door locations. Players landing here roll D6; on 1 they fall through.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub trap_doors: Vec<FieldCoordinate>,

    /// Java: FieldModel.fTargetSelectionState
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_selection_state: Option<TargetSelectionState>,

    /// Java: FieldModel.chomped — maps chomper player id → list of chompee player ids.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub chomped: HashMap<PlayerId, Vec<PlayerId>>,

    /// Java: FieldModel.fMultiBlockTargets — player ids selected as multi-block targets.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub multi_block_targets: Vec<PlayerId>,

    /// Java: FieldModel.fMultiBlockTargetCoordinates — coordinates of multi-block targets.
    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    pub multi_block_target_coordinates: HashSet<FieldCoordinate>,

    /// Java: FieldModel.fBlockedForTricksterCoordinates — squares blocked for trickster.
    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    pub blocked_for_trickster_coordinates: HashSet<FieldCoordinate>,

    /// Java: FieldModel.fDiceDecorations — transient dice display decorations.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dice_decorations: Vec<DiceDecoration>,

    /// Java: FieldModel.fCardEffectsByPlayerId — active card effects per player.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub card_effects: HashMap<PlayerId, HashSet<CardEffect>>,

    /// Java: addPrayerEnhancements / removePrayerEnhancements — prayer name → player IDs with that prayer active.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub prayer_enhancements: HashMap<String, HashSet<PlayerId>>,

    /// Java: FieldModel.fCardsByPlayerId — cards currently assigned to players.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub player_cards: HashMap<PlayerId, Vec<Card>>,

    /// Java: FieldModel.fPlayerMarkers — coach-authored text markers per player.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub player_markers: Vec<PlayerMarker>,

    /// Java: FieldModel.fFieldMarkers — coach-authored text markers per pitch square.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub field_markers: Vec<FieldMarker>,

    /// Java: FieldModel.fBloodspots (a List). Client-side display data the engine never writes;
    /// carried for the replay JSON (BACKLOG §O).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bloodspots: Vec<BloodSpot>,

    /// Java: FieldModel.fTrackNumbers (a HashSet keyed by coordinate). Client-side display data the
    /// engine never writes; carried for the replay JSON (BACKLOG §O).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub track_numbers: Vec<TrackNumber>,
}

impl FieldModel {
    pub fn new() -> Self {
        FieldModel {
            weather: Weather::Nice,
            ball_coordinate: None,
            ball_moving: false,
            ball_in_play: false,
            bomb_coordinate: None,
            bomb_moving: false,
            out_of_bounds: false,
            player_coordinates: HashMap::new(),
            coordinate_arrival: HashMap::new(),
            arrival_counter: 0,
            player_states: HashMap::new(),
            range_ruler: None,
            move_squares: HashMap::new(),
            pushback_squares: Vec::new(),
            trap_doors: Vec::new(),
            target_selection_state: None,
            chomped: HashMap::new(),
            multi_block_targets: Vec::new(),
            multi_block_target_coordinates: HashSet::new(),
            blocked_for_trickster_coordinates: HashSet::new(),
            dice_decorations: Vec::new(),
            card_effects: HashMap::new(),
            prayer_enhancements: HashMap::new(),
            player_cards: HashMap::new(),
            player_markers: Vec::new(),
            field_markers: Vec::new(),
            bloodspots: Vec::new(),
            track_numbers: Vec::new(),
        }
    }

    /// True if there is a trap door at the given coordinate.
    pub fn has_trap_door(&self, coord: FieldCoordinate) -> bool {
        self.trap_doors.contains(&coord)
    }

    /// Java: FieldModel.add(TrapDoor) — places a trapdoor at the given coordinate.
    pub fn add_trap_door(&mut self, coord: FieldCoordinate) {
        if !self.trap_doors.contains(&coord) {
            self.trap_doors.push(coord);
        }
    }

    /// Java: FieldModel.clearTrapdoors() — removes all trapdoors from the field.
    pub fn clear_trap_doors(&mut self) {
        self.trap_doors.clear();
    }

    pub fn player_coordinate(&self, id: &str) -> Option<FieldCoordinate> {
        self.player_coordinates.get(id).copied()
    }

    pub fn player_state(&self, id: &str) -> Option<PlayerState> {
        self.player_states.get(id).copied()
    }

    pub fn set_player_state(&mut self, id: &str, state: PlayerState) {
        let changed = self.player_states.get(id) != Some(&state);
        self.player_states.insert(id.to_owned(), state);
        // Java: setPlayerState notifies FIELD_MODEL_SET_PLAYER_STATE on a changed state, and the
        // bb2025 ChompRemovalObserver answers `!hasTacklezones()` with removeChomps(player).
        // Chomps only exist in bb2025, so the empty map makes this a no-op elsewhere.
        if changed && !self.chomped.is_empty() && !state.has_tacklezones() {
            let events = self.remove_chomps(id);
            crate::chomp_observer::push(events);
        }
    }

    pub fn set_player_coordinate(&mut self, id: &str, coord: FieldCoordinate) {
        // Java `setPlayerCoordinate`: only a CHANGED coordinate re-appends the player to the
        // square's list (`if (!FieldCoordinate.equals(pCoordinate, oldCoordinate))`).
        let changed = self.player_coordinates.get(id) != Some(&coord);
        if changed {
            self.arrival_counter += 1;
            self.coordinate_arrival.insert(id.to_owned(), self.arrival_counter);
        }
        self.player_coordinates.insert(id.to_owned(), coord);
        // Java: FIELD_MODEL_SET_PLAYER_COORDINATE -> ChompRemovalObserver: a box coordinate
        // removes the player's chomps, any other re-checks adjacency (updateChomps).
        if changed && !self.chomped.is_empty() {
            let events = if coord.is_box_coordinate() {
                self.remove_chomps(id)
            } else {
                self.update_chomps(id)
            };
            crate::chomp_observer::push(events);
        }
    }

    /// Java: FieldModel.updatePlayerAndBallPosition(Player, FieldCoordinate).
    ///
    /// Moves the player AND — if the player was standing on the (non-moving) ball,
    /// i.e. was carrying it — moves the ball with them. Returns true when the ball
    /// position was updated. This is the carrier-movement primitive Java uses in
    /// StepMove/StepGoForIt/StepJump/StepPushback/StepFollowup/StepHitAndRun/
    /// StepRaidingParty/StepDropDivingTackler and the Shadowing/Tentacles behaviours.
    pub fn update_player_and_ball_position(&mut self, id: &str, coord: FieldCoordinate) -> bool {
        let mut ball_position_updated = false;
        let old_position = self.player_coordinate(id);
        if !self.ball_moving && old_position.is_some() && old_position == self.ball_coordinate {
            self.ball_coordinate = Some(coord);
            ball_position_updated = true;
        }
        self.set_player_coordinate(id, coord);
        ball_position_updated
    }

    pub fn remove_player(&mut self, id: &str) {
        self.player_coordinates.remove(id);
        self.player_states.remove(id);
    }

    /// Java: FieldModel.addChomp — register chomper→chompee and set chomped bit on chompee's state.
    pub fn add_chomp(&mut self, chomper_id: &str, chompee_id: &str) {
        let chompees = self.chomped
            .entry(chomper_id.to_string())
            .or_default();
        if !chompees.contains(&chompee_id.to_string()) {
            chompees.push(chompee_id.to_string());
            if let Some(st) = self.player_states.get(chompee_id).copied() {
                self.player_states.insert(chompee_id.to_string(), st.change_chomped(true));
            }
        }
    }

    /// Java: FieldModel.removeChomps(Player chomper) — removes all chomps by chomper,
    /// clears the chomped state bit on each chompee that is no longer chomped by anyone.
    /// Returns map of chompeeId → true if chompee state was cleared, false if still chomped by another.
    pub fn remove_chomps(&mut self, chomper_id: &str) -> Vec<(String, bool)> {
        let chompees = match self.chomped.remove(chomper_id) {
            Some(v) => v,
            None => return vec![],
        };
        let mut result = Vec::new();
        for chompee_id in chompees {
            let still_chomped = self.chomped.values().any(|list| list.contains(&chompee_id));
            if !still_chomped {
                if let Some(st) = self.player_states.get(chompee_id.as_str()).copied() {
                    self.player_states.insert(chompee_id.clone(), st.change_chomped(false));
                }
                result.push((chompee_id, true));
            } else {
                result.push((chompee_id, false));
            }
        }
        result
    }

    /// Java: FieldModel.updateChomps(Player chomper) — removes any chomps where the chompee
    /// is no longer adjacent to the chomper.
    pub fn update_chomps(&mut self, chomper_id: &str) -> Vec<(String, bool)> {
        let chomper_coord = match self.player_coordinates.get(chomper_id).copied() {
            Some(c) => c,
            None => return self.remove_chomps(chomper_id),
        };
        let to_remove: Vec<String> = match self.chomped.get(chomper_id) {
            Some(chompees) => chompees.iter()
                .filter(|chompee_id| {
                    let adjacent = self.player_coordinates.get(chompee_id.as_str())
                        .map_or(false, |c| chomper_coord.is_adjacent(*c));
                    !adjacent
                })
                .cloned()
                .collect(),
            None => return vec![],
        };
        let mut result = Vec::new();
        for chompee_id in to_remove {
            let removed = self.remove_single_chomp(chomper_id, &chompee_id);
            result.push((chompee_id, removed));
        }
        result
    }

    /// Java: FieldModel.removeChomp(Player chomper, Player chompee) — remove a single chomp entry.
    /// Returns true if the chompee's state was cleared (no longer chomped by anyone).
    pub fn remove_single_chomp(&mut self, chomper_id: &str, chompee_id: &str) -> bool {
        if let Some(chompees) = self.chomped.get_mut(chomper_id) {
            if let Some(pos) = chompees.iter().position(|id| id == chompee_id) {
                chompees.remove(pos);
                let still_chomped = self.chomped.values().any(|list| list.contains(&chompee_id.to_string()));
                if !still_chomped {
                    if let Some(st) = self.player_states.get(chompee_id).copied() {
                        self.player_states.insert(chompee_id.to_string(), st.change_chomped(false));
                    }
                    return true;
                }
            }
        }
        false
    }

    pub fn player_at(&self, coord: FieldCoordinate) -> Option<&PlayerId> {
        // Java `getPlayer(coordinate)`: the LAST player to arrive on the square.
        self.player_coordinates
            .iter()
            .filter(|(_, &c)| c == coord)
            .max_by_key(|(id, _)| self.coordinate_arrival.get(*id).copied().unwrap_or(0))
            .map(|(id, _)| id)
    }

    /// All player ids currently on the pitch (not in dugout).
    pub fn players_on_pitch(&self) -> impl Iterator<Item = &PlayerId> {
        self.player_coordinates
            .iter()
            .filter(|(_, coord)| coord.is_on_pitch())
            .map(|(id, _)| id)
    }

    /// Java: FieldModel.isMultiBlockTarget — returns true if the player is a multi-block target.
    pub fn is_multi_block_target(&self, player_id: &str) -> bool {
        self.multi_block_targets.iter().any(|id| id == player_id)
    }

    /// Java: FieldModel.wasMultiBlockTargetSquare — returns true if the coordinate was a multi-block target square.
    pub fn was_multi_block_target_square(&self, coord: FieldCoordinate) -> bool {
        self.multi_block_target_coordinates.contains(&coord)
    }

    /// Java: FieldModel.replaceMultiBlockTargetCoordinate — replaces old with new if present.
    pub fn replace_multi_block_target_coordinate(&mut self, old: FieldCoordinate, new: FieldCoordinate) {
        if self.multi_block_target_coordinates.remove(&old) {
            self.multi_block_target_coordinates.insert(new);
        }
    }

    /// Java: FieldModel.isBlockedForTrickster
    pub fn is_blocked_for_trickster(&self, coord: FieldCoordinate) -> bool {
        self.blocked_for_trickster_coordinates.contains(&coord)
    }

    /// Java: FieldModel.clearMultiBlockTargets — clears all multi-block state and resets player state bits.
    pub fn clear_multi_block_targets(&mut self) {
        let targets: Vec<PlayerId> = self.multi_block_targets.clone();
        for target in &targets {
            if let Some(state) = self.player_states.get(target).copied() {
                self.player_states.insert(
                    target.clone(),
                    state.change_selected_stab_target(false).change_selected_block_target(false),
                );
            }
        }
        self.multi_block_targets.clear();
        self.multi_block_target_coordinates.clear();
        self.blocked_for_trickster_coordinates.clear();
    }

    /// Java: FieldModel.clearDiceDecorations
    pub fn clear_dice_decorations(&mut self) {
        self.dice_decorations.clear();
    }

    /// Java: FieldModel.add(DiceDecoration)
    pub fn add_dice_decoration(&mut self, d: DiceDecoration) {
        self.dice_decorations.push(d);
    }

    /// Java: FieldModel.remove(DiceDecoration)
    pub fn remove_dice_decoration(&mut self, d: &DiceDecoration) -> bool {
        if let Some(pos) = self.dice_decorations.iter().position(|x| x == d) {
            self.dice_decorations.remove(pos);
            true
        } else {
            false
        }
    }

    /// Java: FieldModel.getDiceDecorations
    pub fn get_dice_decorations(&self) -> &[DiceDecoration] {
        &self.dice_decorations
    }

    /// Java: FieldModel.getDiceDecoration(FieldCoordinate) — find decoration at a coordinate.
    pub fn get_dice_decoration_at(&self, coord: FieldCoordinate) -> Option<&DiceDecoration> {
        self.dice_decorations.iter().find(|d| d.coordinate == Some(coord))
    }

    /// Java: FieldModel.clearMoveSquares
    pub fn clear_move_squares(&mut self) {
        self.move_squares.clear();
    }

    /// Java: FieldModel.getMoveSquare(FieldCoordinate)
    pub fn get_move_square(&self, coord: FieldCoordinate) -> Option<MoveSquare> {
        self.move_squares.get(&coord).copied()
    }

    /// Java: FieldModel.add(MoveSquare) — inserts with coordinate as key.
    pub fn add_move_square(&mut self, ms: MoveSquare) {
        self.move_squares.insert(ms.coordinate, ms);
    }

    /// Adjacent coordinates within the field bounds.
    pub fn adjacent_on_pitch(&self, coord: FieldCoordinate) -> Vec<FieldCoordinate> {
        coord
            .neighbours()
            .iter()
            .copied()
            .filter(|c| c.is_on_pitch())
            .collect()
    }

    /// Java: FieldModel.addCardEffect — adds a card effect to a player.
    pub fn add_card_effect(&mut self, player_id: &str, effect: CardEffect) {
        self.card_effects
            .entry(player_id.to_string())
            .or_default()
            .insert(effect);
    }

    /// Java: FieldModel.removeCardEffect — removes a card effect from a player. Returns true if removed.
    pub fn remove_card_effect(&mut self, player_id: &str, effect: CardEffect) -> bool {
        if let Some(effects) = self.card_effects.get_mut(player_id) {
            let removed = effects.remove(&effect);
            if effects.is_empty() {
                self.card_effects.remove(player_id);
            }
            removed
        } else {
            false
        }
    }

    /// Java: FieldModel.hasCardEffect — checks if a player has a specific card effect.
    pub fn has_card_effect(&self, player_id: &str, effect: CardEffect) -> bool {
        self.card_effects
            .get(player_id)
            .map_or(false, |set| set.contains(&effect))
    }

    /// Java: FieldModel.findPlayers(CardEffect) — returns all player ids with the given card effect.
    pub fn find_players_with_card_effect(&self, effect: CardEffect) -> Vec<&str> {
        self.card_effects
            .iter()
            .filter(|(_, effects)| effects.contains(&effect))
            .map(|(id, _)| id.as_str())
            .collect()
    }

    /// Java: FieldModel.addCard(Player, Card) — assigns a card to a player on the field.
    pub fn add_card(&mut self, player_id: &str, card: Card) {
        self.player_cards
            .entry(player_id.to_string())
            .or_default()
            .push(card);
    }

    /// Java: FieldModel.removeCard(Player, Card) — removes a card from a player.
    pub fn remove_card(&mut self, player_id: &str, card_name: &str) -> Option<Card> {
        if let Some(cards) = self.player_cards.get_mut(player_id) {
            if let Some(pos) = cards.iter().position(|c| c.name == card_name) {
                let removed = cards.remove(pos);
                if cards.is_empty() {
                    self.player_cards.remove(player_id);
                }
                return Some(removed);
            }
        }
        None
    }

    /// Java: FieldModel.getCards(Player) — returns all cards assigned to a player.
    pub fn get_cards(&self, player_id: &str) -> &[Card] {
        self.player_cards
            .get(player_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Java: FieldModel.findPlayer(Card) — finds which player holds a given card.
    pub fn find_player_with_card(&self, card_name: &str) -> Option<&str> {
        self.player_cards
            .iter()
            .find(|(_, cards)| cards.iter().any(|c| c.name == card_name))
            .map(|(id, _)| id.as_str())
    }

    /// Java: FieldModel.keepDeactivatedCard(Player, Card) — card stays on field after deactivation.
    pub fn keep_deactivated_card(&mut self, player_id: &str, card_name: &str) {
        // Card already in player_cards — mark it deactivated (it stays assigned).
        // In the headless engine the distinction doesn't affect game state.
        let _ = (player_id, card_name);
    }

    /// Java: FieldModel.addPrayerEnhancements(Player, Prayer) — marks the prayer as active on the player.
    pub fn add_prayer_enhancement(&mut self, player_id: &str, prayer_name: &str) {
        self.prayer_enhancements
            .entry(prayer_name.to_string())
            .or_default()
            .insert(player_id.to_string());
    }

    /// Java: FieldModel.removePrayerEnhancements(Player, Prayer) — removes the prayer from the player.
    pub fn remove_prayer_enhancement(&mut self, player_id: &str, prayer_name: &str) {
        if let Some(players) = self.prayer_enhancements.get_mut(prayer_name) {
            players.remove(player_id);
            if players.is_empty() {
                self.prayer_enhancements.remove(prayer_name);
            }
        }
    }

    /// Returns true if the given prayer is currently active on the given player.
    pub fn has_prayer_enhancement(&self, player_id: &str, prayer_name: &str) -> bool {
        self.prayer_enhancements
            .get(prayer_name)
            .map_or(false, |players| players.contains(player_id))
    }

    /// Returns all player IDs that currently have the given prayer active.
    pub fn find_players_with_prayer_enhancement(&self, prayer_name: &str) -> Vec<&str> {
        self.prayer_enhancements
            .get(prayer_name)
            .map(|ids| ids.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    /// Removes all prayer enhancements for the given prayer from all players.
    pub fn clear_prayer_enhancement(&mut self, prayer_name: &str) {
        self.prayer_enhancements.remove(prayer_name);
    }

    /// Java: FieldModel.clearTrackNumbers() — clears the client-side movement-track visualisation.
    /// No game-state effect; track numbers are purely a UI concern.
    pub fn clear_track_numbers(&mut self) {
        // No-op: track numbers are client-side visualisation data not stored in the engine model.
    }

    /// Java: FieldModel.getPlayerMarkers()
    pub fn get_player_markers(&self) -> &[PlayerMarker] {
        &self.player_markers
    }

    /// Java: FieldModel.getPlayerMarker(String pPlayerId)
    pub fn get_player_marker(&self, player_id: &str) -> Option<&PlayerMarker> {
        self.player_markers
            .iter()
            .find(|m| m.get_player_id() == Some(player_id))
    }

    /// Java: FieldModel.add(PlayerMarker) — replaces any existing marker for the same player.
    pub fn add_player_marker(&mut self, player_marker: PlayerMarker) {
        let player_id = player_marker.get_player_id().map(|s| s.to_string());
        self.player_markers.retain(|m| m.get_player_id().map(|s| s.to_string()) != player_id);
        self.player_markers.push(player_marker);
    }

    /// Java: FieldModel.remove(PlayerMarker)
    pub fn remove_player_marker(&mut self, player_id: &str) -> bool {
        let before = self.player_markers.len();
        self.player_markers.retain(|m| m.get_player_id() != Some(player_id));
        self.player_markers.len() != before
    }

    /// Java: FieldModel.getFieldMarker(FieldCoordinate)
    pub fn get_field_marker(&self, coordinate: FieldCoordinate) -> Option<&FieldMarker> {
        self.field_markers
            .iter()
            .find(|m| m.get_coordinate() == Some(&coordinate))
    }

    /// Java: FieldModel.add(FieldMarker) — replaces any existing marker at the same coordinate.
    pub fn add_field_marker(&mut self, field_marker: FieldMarker) {
        let coordinate = field_marker.coordinate;
        self.field_markers.retain(|m| m.coordinate != coordinate);
        self.field_markers.push(field_marker);
    }

    /// Java: FieldModel.remove(FieldMarker)
    pub fn remove_field_marker(&mut self, coordinate: FieldCoordinate) -> bool {
        let before = self.field_markers.len();
        self.field_markers.retain(|m| m.get_coordinate() != Some(&coordinate));
        self.field_markers.len() != before
    }
}

/// Java `FieldModel` JSON (BACKLOG §O replay states). The game's players come from its two teams:
/// `getGame().getPlayerById` = home first, then away; `getGame().getPlayers()` = home players then
/// away players, each in `Team.getPlayers()` (nr) order.
impl FieldModel {
    /// Java: `FieldModel.toJsonValue()`.
    pub fn to_json_value(
        &self,
        rules: crate::enums::Rules,
        team_home: &crate::model::team::Team,
        team_away: &crate::model::team::Team,
    ) -> Result<serde_json::Value, String> {
        use crate::util::java_json as j;
        use serde_json::Value;
        let mut o = j::JsonObject::new();
        o.insert("weather".into(), Value::from(self.weather.name()));
        o.insert("ballCoordinate".into(), j::field_coordinate_or_null(self.ball_coordinate));
        o.insert("ballInPlay".into(), Value::Bool(self.ball_in_play));
        o.insert("ballMoving".into(), Value::Bool(self.ball_moving));
        o.insert("bombCoordinate".into(), j::field_coordinate_or_null(self.bomb_coordinate));
        o.insert("bombMoving".into(), Value::Bool(self.bomb_moving));
        o.insert("bloodspotArray".into(), Value::Array(self.bloodspots.iter().map(BloodSpot::to_json_value).collect()));
        o.insert(
            "pushbackSquareArray".into(),
            Value::Array(self.pushback_squares.iter().map(PushbackSquare::to_json_value).collect()),
        );
        o.insert("moveSquareArray".into(), Value::Array(self.move_squares.values().map(MoveSquare::to_json_value).collect()));
        o.insert(
            "trackNumberArray".into(),
            Value::Array(self.track_numbers.iter().map(TrackNumber::to_json_value).collect()),
        );
        o.insert(
            "diceDecorationArray".into(),
            Value::Array(self.dice_decorations.iter().map(DiceDecoration::to_json_value).collect()),
        );
        o.insert(
            "fieldMarkerArray".into(),
            Value::Array(self.field_markers.iter().map(FieldMarker::to_json_value).collect()),
        );
        o.insert(
            "playerMarkerArray".into(),
            Value::Array(self.player_markers.iter().map(PlayerMarker::to_json_value).collect()),
        );

        let mut player_data_array = Vec::new();
        for team in [team_home, team_away] {
            let mut players: Vec<&crate::model::player::Player> = team.players.iter().collect();
            players.sort_by_key(|p| p.nr);
            for player in players {
                let mut data = j::JsonObject::new();
                data.insert("playerId".into(), Value::from(player.id.clone()));
                data.insert("playerCoordinate".into(), j::field_coordinate_or_null(self.player_coordinate(&player.id)));
                // getPlayerState: an absent (or null) state reads as PlayerState.UNKNOWN (0).
                let state = self.player_state(&player.id).map_or(0, |s| s.0 as i32);
                data.insert("playerState".into(), Value::from(state));
                let cards = self.get_cards(&player.id);
                if !cards.is_empty() {
                    return Err(format!("FieldModel.toJsonValue: cards of {} not ported yet", player.id));
                }
                data.insert("cards".into(), Value::Array(Vec::new()));
                if self.card_effects.get(&player.id).is_some_and(|e| !e.is_empty()) {
                    return Err(format!("FieldModel.toJsonValue: card effects of {} not ported yet", player.id));
                }
                data.insert("cardEffects".into(), Value::Array(Vec::new()));
                player_data_array.push(Value::Object(data));
            }
        }
        o.insert("playerDataArray".into(), Value::Array(player_data_array));

        if let Some(state) = &self.target_selection_state {
            o.insert("targetSelectionState".into(), state.to_json_value(rules)?);
        }
        let trap_doors = self
            .trap_doors
            .iter()
            .map(|c| {
                let mut t = j::JsonObject::new();
                t.insert("coordinate".into(), j::field_coordinate_or_null(Some(*c)));
                Value::Object(t)
            })
            .collect();
        o.insert("trapDoors".into(), Value::Array(trap_doors));
        o.insert("outOfBounds".into(), Value::Bool(self.out_of_bounds));
        if let Some(range_ruler) = &self.range_ruler {
            o.insert("rangeRuler".into(), range_ruler.to_json_value());
        }
        // JsonStringListMapOption.addTo: one JsonStringArrayOption per entry.
        let mut chomped = j::JsonObject::new();
        for (chomper, chompees) in &self.chomped {
            chomped.insert(chomper.clone(), Value::Array(chompees.iter().map(|c| Value::from(c.clone())).collect()));
        }
        o.insert("chomped".into(), Value::Object(chomped));
        Ok(Value::Object(o))
    }

    /// Java: `FieldModel.initFrom(IFactorySource, JsonValue)`, 1:1. The `HashSet` fields keep the
    /// FIRST of two elements with the same key (coordinate / player id: `HashSet.add` does not
    /// replace); player coordinates, states, cards and effects only attach to a player the game
    /// holds. `Err` where Java throws and where the Rust model cannot hold the Java value (null or
    /// unknown weather, cards / card effects -- none occur in the corpus's start states yet).
    pub fn init_from(
        &mut self,
        rules: crate::enums::Rules,
        team_home: &crate::model::team::Team,
        team_away: &crate::model::team::Team,
        json: &serde_json::Value,
    ) -> Result<(), String> {
        use crate::util::java_json as j;
        let o = j::to_json_object(json)?;
        let array = |key: &str| -> Result<&Vec<serde_json::Value>, String> {
            j::get_array(o, key)?.ok_or_else(|| format!("FieldModel: {key} is null (NPE)"))
        };

        let weather = j::get_enum_name(o, "weather")?;
        self.weather = weather
            .and_then(Weather::for_name_ignore_case)
            .ok_or_else(|| format!("FieldModel: weather {weather:?} is null in Java"))?;
        self.ball_coordinate = j::get_field_coordinate(o, "ballCoordinate")?;
        self.ball_in_play = j::get_boolean(o, "ballInPlay")?;
        self.ball_moving = j::get_boolean(o, "ballMoving")?;
        self.bomb_coordinate = j::get_field_coordinate(o, "bombCoordinate")?;
        self.bomb_moving = j::get_boolean(o, "bombMoving")?;

        self.bloodspots.clear();
        for element in array("bloodspotArray")? {
            self.bloodspots.push(BloodSpot::init_from(element)?);
        }
        self.pushback_squares.clear();
        for element in array("pushbackSquareArray")? {
            let square = PushbackSquare::init_from(element)?;
            if !self.pushback_squares.iter().any(|s| s.coordinate == square.coordinate) {
                self.pushback_squares.push(square);
            }
        }
        self.move_squares.clear();
        for element in array("moveSquareArray")? {
            let square = MoveSquare::init_from(element)?;
            self.move_squares.entry(square.coordinate).or_insert(square);
        }
        self.track_numbers.clear();
        for element in array("trackNumberArray")? {
            let number = TrackNumber::init_from(element)?;
            if number.coordinate.is_none() {
                return Err("TrackNumber: null coordinate (hashCode NPE)".into());
            }
            if !self.track_numbers.iter().any(|t| t.coordinate == number.coordinate) {
                self.track_numbers.push(number);
            }
        }
        self.dice_decorations.clear();
        for element in array("diceDecorationArray")? {
            let decoration = DiceDecoration::init_from(element)?;
            if decoration.coordinate.is_none() {
                return Err("DiceDecoration: null coordinate (hashCode NPE)".into());
            }
            if !self.dice_decorations.iter().any(|d| d.coordinate == decoration.coordinate) {
                self.dice_decorations.push(decoration);
            }
        }
        self.field_markers.clear();
        for element in array("fieldMarkerArray")? {
            let marker = FieldMarker::init_from(element)?;
            if !self.field_markers.iter().any(|m| m.coordinate == marker.coordinate) {
                self.field_markers.push(marker);
            }
        }
        self.player_markers.clear();
        for element in array("playerMarkerArray")? {
            let marker = PlayerMarker::init_from(element)?;
            if !self.player_markers.iter().any(|m| m.player_id == marker.player_id) {
                self.player_markers.push(marker);
            }
        }

        self.player_coordinates.clear();
        self.coordinate_arrival.clear();
        self.player_states.clear();
        self.player_cards.clear();
        self.trap_doors.clear();

        for element in array("playerDataArray")? {
            let data = j::to_json_object(element)?;
            let player_id = j::get_string(data, "playerId")?;
            // getGame().getPlayerById: null for a null id or a player neither team holds; every
            // setter below is then a no-op.
            let known = player_id.as_deref().is_some_and(|id| team_home.has_player(id) || team_away.has_player(id));
            let coordinate = j::get_field_coordinate(data, "playerCoordinate")?;
            let state = j::get_player_state(data, "playerState")?;
            let cards = j::get_string_array(data, "cards")?;
            let card_effects = j::get_string_array(data, "cardEffects")?;
            let Some(id) = player_id.filter(|_| known) else { continue };
            // setPlayerCoordinate: a null coordinate is ignored; only a changed one is stored.
            if let Some(coordinate) = coordinate {
                if self.player_coordinates.get(&id) != Some(&coordinate) {
                    self.arrival_counter += 1;
                    self.coordinate_arrival.insert(id.clone(), self.arrival_counter);
                    self.player_coordinates.insert(id.clone(), coordinate);
                }
            }
            // setPlayerState: stored when there is no state yet or the id changes (a null state
            // over no state stores null = absent here, which reads back as UNKNOWN like Java's).
            if let Some(state) = state {
                if self.player_states.get(&id) != Some(&state) {
                    self.player_states.insert(id.clone(), state);
                }
            }
            if cards.is_some_and(|c| !c.is_empty()) {
                return Err(format!("FieldModel.initFrom: cards of {id} not ported yet"));
            }
            if card_effects.is_some_and(|c| !c.is_empty()) {
                return Err(format!("FieldModel.initFrom: card effects of {id} not ported yet"));
            }
        }

        let mut target_selection = j::get_object(o, "targetSelectionState")?;
        if target_selection.is_none() {
            target_selection = j::get_object(o, "blitzState")?;
        }
        if let Some(state) = target_selection {
            self.target_selection_state =
                Some(TargetSelectionState::init_from(rules, &serde_json::Value::Object(state.clone()))?);
        }

        if let Some(trap_doors) = j::get_array(o, "trapDoors")? {
            for element in trap_doors {
                // TrapDoor.initFrom; the List keeps duplicates.
                let coordinate = j::get_field_coordinate(j::to_json_object(element)?, "coordinate")?
                    .ok_or("TrapDoor: null coordinate (not representable)")?;
                self.trap_doors.push(coordinate);
            }
        }

        if j::is_defined_in(o, "outOfBounds") {
            self.out_of_bounds = j::get_boolean(o, "outOfBounds")?;
        }
        if j::is_defined_in(o, "rangeRuler") {
            let ruler = j::get_object(o, "rangeRuler")?.ok_or("FieldModel: rangeRuler null (NPE)")?;
            self.range_ruler = Some(RangeRuler::init_from(&serde_json::Value::Object(ruler.clone()))?);
        }
        if j::is_defined_in(o, "chomped") {
            // JsonStringListMapOption.getFrom (a non-object value yields an empty map), putAll.
            if let Some(map) = o.get("chomped").and_then(serde_json::Value::as_object) {
                for (chomper, list) in map {
                    let chompees = j::get_string_array(map, chomper)?
                        .filter(|l| !l.is_empty())
                        .ok_or_else(|| format!("chomped.{chomper}: {list} (Java stores null)"))?
                        .into_iter()
                        .map(|c| c.ok_or_else(|| format!("chomped.{chomper}: null element")))
                        .collect::<Result<Vec<_>, _>>()?;
                    self.chomped.insert(chomper.clone(), chompees);
                }
            }
        }
        Ok(())
    }
}

impl Default for FieldModel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod json_tests {
    use super::*;
    use crate::enums::Rules;
    use crate::model::player::Player;
    use crate::model::team::Team;
    use serde_json::json;

    fn team(ids: &[(&str, i32)]) -> Team {
        let mut t = Team::new();
        for (id, nr) in ids {
            t.players.push(Player { id: id.to_string(), nr: *nr, ..Default::default() });
        }
        t
    }

    fn base() -> serde_json::Value {
        json!({
            "weather": "Nice Weather", "ballCoordinate": [24, 8], "ballInPlay": true, "ballMoving": false,
            "bombCoordinate": null, "bombMoving": false,
            "bloodspotArray": [{"coordinate": [13, 7], "injury": 8}, {"coordinate": [13, 7], "injury": 5}],
            "pushbackSquareArray": [], "moveSquareArray": [], "trackNumberArray": [], "diceDecorationArray": [],
            "fieldMarkerArray": [], "playerMarkerArray": [],
            "playerDataArray": [], "trapDoors": [], "outOfBounds": false, "chomped": {}
        })
    }

    #[test]
    fn json_field_model_round_trip_java_shape() {
        // Shapes copied from the Java start dumps (D:/ffb_replay_parity/start_all).
        let (home, away) = (team(&[("h2", 2), ("h1", 1)]), team(&[("a1", 1)]));
        let mut j = base();
        j["playerDataArray"] = json!([
            {"playerId": "h1", "playerCoordinate": [12, 7], "playerState": 1, "cards": [], "cardEffects": []},
            {"playerId": "a1", "playerCoordinate": null, "playerState": 258, "cards": null, "cardEffects": []},
            {"playerId": "nobody", "playerCoordinate": [1, 1], "playerState": 1, "cards": [], "cardEffects": []}
        ]);
        j["trapDoors"] = json!([{"coordinate": [19, 13]}, {"coordinate": [19, 13]}]);
        j["targetSelectionState"] = json!({"playerId": "h1", "playerStateOld": 258, "targetSelectionStatus": "SELECTED",
            "targetSelectionStatusIsCommitted": true, "usedSkills": []});
        let mut fm = FieldModel::new();
        fm.init_from(Rules::Bb2020, &home, &away, &j).unwrap();
        let out = fm.to_json_value(Rules::Bb2020, &home, &away).unwrap();
        // Bloodspots are a List (duplicates kept, order kept); trap doors a List too.
        assert_eq!(out["bloodspotArray"], j["bloodspotArray"]);
        assert_eq!(out["trapDoors"], j["trapDoors"]);
        // getGame().getPlayers(): home by nr, then away; an unknown id attaches nowhere; a player
        // with no state reads as UNKNOWN (0); an absent coordinate is null.
        assert_eq!(
            out["playerDataArray"],
            json!([
                {"playerId": "h1", "playerCoordinate": [12, 7], "playerState": 1, "cards": [], "cardEffects": []},
                {"playerId": "h2", "playerCoordinate": null, "playerState": 0, "cards": [], "cardEffects": []},
                {"playerId": "a1", "playerCoordinate": null, "playerState": 258, "cards": [], "cardEffects": []}
            ])
        );
        assert_eq!(out["targetSelectionState"], j["targetSelectionState"]);
        assert_eq!(out["weather"], json!("Nice Weather"));
        assert_eq!(out["ballCoordinate"], json!([24, 8]));
        assert!(out.get("rangeRuler").is_none());
    }

    #[test]
    fn json_field_model_hash_sets_keep_the_first_element() {
        let (home, away) = (team(&[]), team(&[]));
        let mut j = base();
        j["moveSquareArray"] = json!([
            {"coordinate": [16, 11], "minimumRollDodge": 3, "minimumRollGfi": 0},
            {"coordinate": [16, 11], "minimumRollDodge": 0, "minimumRollGfi": 2}
        ]);
        j["playerMarkerArray"] = json!([
            {"playerId": "p", "homeText": "first", "awayText": ""},
            {"playerId": "p", "homeText": "second", "awayText": ""}
        ]);
        j["pushbackSquareArray"] = json!([
            {"coordinate": [5, 10], "direction": "west", "homeChoice": false, "locked": false, "selected": true},
            {"coordinate": [5, 10], "direction": "South", "homeChoice": false, "locked": false, "selected": false}
        ]);
        let mut fm = FieldModel::new();
        fm.init_from(Rules::Bb2025, &home, &away, &j).unwrap();
        let out = fm.to_json_value(Rules::Bb2025, &home, &away).unwrap();
        assert_eq!(out["moveSquareArray"], json!([{"coordinate": [16, 11], "minimumRollDodge": 3, "minimumRollGfi": 0}]));
        assert_eq!(out["playerMarkerArray"], json!([{"playerId": "p", "homeText": "first", "awayText": ""}]));
        // DirectionFactory.forName is equalsIgnoreCase; getName() writes the canonical name.
        assert_eq!(
            out["pushbackSquareArray"],
            json!([{"coordinate": [5, 10], "direction": "West", "homeChoice": false, "locked": false, "selected": true}])
        );
    }

    #[test]
    fn json_field_model_java_option_rules() {
        let (home, away) = (team(&[("p", 1)]), team(&[]));
        // Legacy blitzState / blitzStatus keys.
        let mut j = base();
        j["blitzState"] = json!({"playerId": "p", "blitzStatus": "STARTED"});
        let mut fm = FieldModel::new();
        fm.init_from(Rules::Bb2016, &home, &away, &j).unwrap();
        let tss = fm.target_selection_state.as_ref().unwrap();
        assert!(tss.is_started() && !tss.is_committed());
        // outOfBounds / rangeRuler / chomped are read only when present.
        let mut j = base();
        j.as_object_mut().unwrap().remove("outOfBounds");
        j["rangeRuler"] = json!({"throwerId": "p", "targetCoordinate": [3, 4], "minimumRoll": 4, "throwTeamMate": false});
        j["chomped"] = json!({"p": ["q"]});
        let mut fm = FieldModel::new();
        fm.out_of_bounds = true;
        fm.init_from(Rules::Bb2025, &home, &away, &j).unwrap();
        assert!(fm.out_of_bounds);
        let out = fm.to_json_value(Rules::Bb2025, &home, &away).unwrap();
        assert_eq!(out["rangeRuler"], j["rangeRuler"]);
        assert_eq!(out["chomped"], json!({"p": ["q"]}));
        // Where Java throws or holds what Rust cannot: Err.
        for (key, value) in [
            ("weather", json!("Acid Rain")),
            ("ballInPlay", serde_json::Value::Null),
            ("bloodspotArray", serde_json::Value::Null),
            ("moveSquareArray", json!([{"coordinate": null, "minimumRollDodge": 0, "minimumRollGfi": 0}])),
            ("diceDecorationArray", json!([{"coordinate": [1, 1], "nrOfDice": 1, "blockKind": "SHOVE"}])),
            ("playerDataArray", json!([{"playerId": "p", "playerCoordinate": null, "playerState": 1, "cards": ["Custard Pie"], "cardEffects": []}])),
        ] {
            let mut j = base();
            j[key] = value;
            assert!(FieldModel::new().init_from(Rules::Bb2025, &home, &away, &j).is_err(), "{key}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::{PlayerState, PS_STANDING};
    use crate::types::FieldCoordinate;
    use crate::model::block_kind::BlockKind;

    #[test]
    fn set_and_get_player_position() {
        let mut fm = FieldModel::new();
        let coord = FieldCoordinate::new(5, 7);
        fm.set_player_coordinate("p1", coord);
        fm.set_player_state("p1", PlayerState(0x00001));
        assert_eq!(fm.player_coordinate("p1"), Some(coord));
        assert_eq!(fm.player_state("p1").map(|s| s.0), Some(0x00001));
    }

    #[test]
    fn player_at_returns_id() {
        let mut fm = FieldModel::new();
        let coord = FieldCoordinate::new(10, 7);
        fm.set_player_coordinate("p1", coord);
        assert_eq!(fm.player_at(coord).map(|s| s.as_str()), Some("p1"));
        assert!(fm.player_at(FieldCoordinate::new(0, 0)).is_none());
    }

    /// Java `FieldModel.getPlayer(coordinate)` returns the LAST player to arrive on a shared
    /// square; re-setting the same coordinate does not reorder (star tier: a shadower stepping
    /// onto a standing Treeman's square, halfling vs chaos_dwarf bb2025 seed 1 — Rust's HashMap
    /// scan flipped between the two players run to run).
    #[test]
    fn player_at_returns_the_last_arrival_on_a_shared_square() {
        for _ in 0..20 {
            let mut fm = FieldModel::new();
            let coord = FieldCoordinate::new(12, 10);
            for i in 0..8 {
                fm.set_player_coordinate(&format!("filler{i}"), FieldCoordinate::new(i, 0));
            }
            fm.set_player_coordinate("treeman", coord);
            fm.set_player_coordinate("shadower", FieldCoordinate::new(12, 11));
            fm.set_player_coordinate("shadower", coord);
            fm.set_player_coordinate("treeman", coord); // unchanged: no re-append
            assert_eq!(fm.player_at(coord).map(|s| s.as_str()), Some("shadower"));
        }
    }

    #[test]
    fn serde_round_trip() {
        let fm = FieldModel::new();
        let json = serde_json::to_string(&fm).unwrap();
        let back: FieldModel = serde_json::from_str(&json).unwrap();
        assert_eq!(fm.weather, back.weather);
    }

    #[test]
    fn ball_coordinate_set_and_get() {
        let mut fm = FieldModel::new();
        assert!(fm.ball_coordinate.is_none());
        fm.ball_coordinate = Some(FieldCoordinate::new(8, 5));
        assert_eq!(fm.ball_coordinate, Some(FieldCoordinate::new(8, 5)));
    }

    #[test]
    fn ball_in_play_flag_toggled() {
        let mut fm = FieldModel::new();
        assert!(!fm.ball_in_play);
        fm.ball_in_play = true;
        assert!(fm.ball_in_play);
    }

    #[test]
    fn players_on_pitch_counts_correctly() {
        let mut fm = FieldModel::new();
        fm.set_player_coordinate("p1", FieldCoordinate::new(5, 7));
        fm.set_player_coordinate("p2", FieldCoordinate::new(10, 3));
        let count = fm.players_on_pitch().count();
        assert_eq!(count, 2);
    }

    #[test]
    fn player_state_missing_returns_none() {
        let fm = FieldModel::new();
        assert!(fm.player_state("nobody").is_none());
    }

    #[test]
    fn has_trap_door_detects_coordinate() {
        let mut fm = FieldModel::new();
        let trap = FieldCoordinate::new(10, 7);
        fm.trap_doors.push(trap);
        assert!(fm.has_trap_door(trap));
        assert!(!fm.has_trap_door(FieldCoordinate::new(11, 7)));
    }

    #[test]
    fn add_trap_door_stores_coordinate() {
        let mut fm = FieldModel::new();
        let coord = FieldCoordinate::new(6, 1);
        fm.add_trap_door(coord);
        assert!(fm.has_trap_door(coord));
    }

    #[test]
    fn add_trap_door_no_duplicate() {
        let mut fm = FieldModel::new();
        let coord = FieldCoordinate::new(6, 1);
        fm.add_trap_door(coord);
        fm.add_trap_door(coord);
        assert_eq!(fm.trap_doors.len(), 1);
    }

    #[test]
    fn clear_trap_doors_empties_vec() {
        let mut fm = FieldModel::new();
        fm.add_trap_door(FieldCoordinate::new(6, 1));
        fm.add_trap_door(FieldCoordinate::new(19, 13));
        assert_eq!(fm.trap_doors.len(), 2);
        fm.clear_trap_doors();
        assert!(fm.trap_doors.is_empty());
    }

    #[test]
    fn remove_player_clears_position_and_state() {
        let mut fm = FieldModel::new();
        let coord = FieldCoordinate::new(5, 7);
        fm.set_player_coordinate("p1", coord);
        fm.set_player_state("p1", PlayerState(0x00001));
        assert!(fm.player_coordinate("p1").is_some());
        fm.remove_player("p1");
        assert!(fm.player_coordinate("p1").is_none());
        assert!(fm.player_state("p1").is_none());
    }

    #[test]
    fn clear_multi_block_targets_clears_all() {
        let mut fm = FieldModel::new();
        fm.multi_block_targets.push("p1".to_string());
        fm.multi_block_targets.push("p2".to_string());
        fm.multi_block_target_coordinates.insert(FieldCoordinate::new(5, 7));
        fm.blocked_for_trickster_coordinates.insert(FieldCoordinate::new(6, 7));

        fm.clear_multi_block_targets();

        assert!(fm.multi_block_targets.is_empty());
        assert!(fm.multi_block_target_coordinates.is_empty());
        assert!(fm.blocked_for_trickster_coordinates.is_empty());
    }

    #[test]
    fn clear_multi_block_targets_clears_player_state_bits() {
        let mut fm = FieldModel::new();
        let state = PlayerState(PS_STANDING)
            .change_selected_block_target(true)
            .change_selected_stab_target(true);
        fm.set_player_state("p1", state);
        fm.multi_block_targets.push("p1".to_string());

        fm.clear_multi_block_targets();

        let after = fm.player_state("p1").unwrap();
        assert!(!after.is_selected_block_target());
        assert!(!after.is_selected_stab_target());
    }

    #[test]
    fn clear_dice_decorations_removes_all() {
        let mut fm = FieldModel::new();
        fm.add_dice_decoration(DiceDecoration::new(FieldCoordinate::new(5, 7), 2, BlockKind::BLOCK));
        fm.add_dice_decoration(DiceDecoration::new(FieldCoordinate::new(6, 7), 1, BlockKind::STAB));
        assert_eq!(fm.get_dice_decorations().len(), 2);

        fm.clear_dice_decorations();
        assert!(fm.get_dice_decorations().is_empty());
    }

    #[test]
    fn add_and_get_dice_decoration() {
        let mut fm = FieldModel::new();
        let d = DiceDecoration::new(FieldCoordinate::new(3, 4), 3, BlockKind::BLOCK);
        fm.add_dice_decoration(d.clone());

        let decorations = fm.get_dice_decorations();
        assert_eq!(decorations.len(), 1);
        assert_eq!(decorations[0], d);
    }

    #[test]
    fn dice_decoration_at_finds_coord() {
        let mut fm = FieldModel::new();
        let coord = FieldCoordinate::new(7, 5);
        let d = DiceDecoration::new(coord, 2, BlockKind::CHAINSAW);
        fm.add_dice_decoration(d.clone());
        fm.add_dice_decoration(DiceDecoration::new(FieldCoordinate::new(1, 1), 1, BlockKind::BLOCK));

        let found = fm.get_dice_decoration_at(coord);
        assert!(found.is_some());
        assert_eq!(found.unwrap().nr_of_dice, 2);

        assert!(fm.get_dice_decoration_at(FieldCoordinate::new(0, 0)).is_none());
    }

    #[test]
    fn add_card_effect_stores_and_queries() {
        let mut fm = FieldModel::new();
        fm.add_card_effect("p1", CardEffect::Distracted);
        assert!(fm.has_card_effect("p1", CardEffect::Distracted));
        assert!(!fm.has_card_effect("p1", CardEffect::Sedative));
        assert!(!fm.has_card_effect("p2", CardEffect::Distracted));
    }

    #[test]
    fn remove_card_effect_clears_entry() {
        let mut fm = FieldModel::new();
        fm.add_card_effect("p1", CardEffect::Sedative);
        assert!(fm.remove_card_effect("p1", CardEffect::Sedative));
        assert!(!fm.has_card_effect("p1", CardEffect::Sedative));
        assert!(!fm.remove_card_effect("p1", CardEffect::Sedative));
    }

    #[test]
    fn find_players_with_card_effect_returns_affected() {
        let mut fm = FieldModel::new();
        fm.add_card_effect("p1", CardEffect::Distracted);
        fm.add_card_effect("p2", CardEffect::Distracted);
        fm.add_card_effect("p3", CardEffect::Sedative);
        let mut distracted = fm.find_players_with_card_effect(CardEffect::Distracted);
        distracted.sort();
        assert_eq!(distracted, vec!["p1", "p2"]);
        let sedated = fm.find_players_with_card_effect(CardEffect::Sedative);
        assert_eq!(sedated, vec!["p3"]);
    }

    #[test]
    fn add_card_assigns_to_player() {
        let mut fm = FieldModel::new();
        let card = Card::new("Chop Block", Some("CHOP_BLOCK"));
        fm.add_card("p1", card);
        let cards = fm.get_cards("p1");
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].get_name(), "Chop Block");
    }

    #[test]
    fn remove_card_removes_and_returns_it() {
        let mut fm = FieldModel::new();
        fm.add_card("p1", Card::new("Witch Brew", Some("WITCH_BREW")));
        fm.add_card("p1", Card::new("Distract", Some("DISTRACT")));
        let removed = fm.remove_card("p1", "Witch Brew");
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().get_name(), "Witch Brew");
        assert_eq!(fm.get_cards("p1").len(), 1);
    }

    #[test]
    fn remove_card_unknown_returns_none() {
        let mut fm = FieldModel::new();
        assert!(fm.remove_card("p1", "No Such Card").is_none());
    }

    #[test]
    fn find_player_with_card_finds_correct_player() {
        let mut fm = FieldModel::new();
        fm.add_card("p1", Card::new("Chop Block", Some("CHOP_BLOCK")));
        fm.add_card("p2", Card::new("Distract", Some("DISTRACT")));
        assert_eq!(fm.find_player_with_card("Chop Block"), Some("p1"));
        assert_eq!(fm.find_player_with_card("Distract"), Some("p2"));
        assert!(fm.find_player_with_card("Unknown").is_none());
    }

    #[test]
    fn remove_card_removes_player_entry_when_empty() {
        let mut fm = FieldModel::new();
        fm.add_card("p1", Card::new("Chop Block", Some("CHOP_BLOCK")));
        fm.remove_card("p1", "Chop Block");
        assert!(fm.get_cards("p1").is_empty());
        assert!(fm.find_player_with_card("Chop Block").is_none());
    }

    #[test]
    fn add_player_marker_stores_and_replaces() {
        let mut fm = FieldModel::new();
        let mut m1 = crate::marking::player_marker::PlayerMarker::with_player_id("p1");
        m1.set_home_text("first");
        fm.add_player_marker(m1);
        assert_eq!(fm.get_player_marker("p1").and_then(|m| m.get_home_text()), Some("first"));

        let mut m2 = crate::marking::player_marker::PlayerMarker::with_player_id("p1");
        m2.set_home_text("second");
        fm.add_player_marker(m2);
        assert_eq!(fm.get_player_markers().len(), 1);
        assert_eq!(fm.get_player_marker("p1").and_then(|m| m.get_home_text()), Some("second"));
    }

    #[test]
    fn remove_player_marker_removes_entry() {
        let mut fm = FieldModel::new();
        fm.add_player_marker(crate::marking::player_marker::PlayerMarker::with_player_id("p1"));
        assert!(fm.remove_player_marker("p1"));
        assert!(fm.get_player_marker("p1").is_none());
        assert!(!fm.remove_player_marker("p1"));
    }

    #[test]
    fn add_field_marker_stores_and_replaces() {
        let mut fm = FieldModel::new();
        let coord = FieldCoordinate::new(3, 3);
        let mut m1 = crate::marking::field_marker::FieldMarker::with_coordinate(coord);
        m1.set_home_text("first");
        fm.add_field_marker(m1);
        assert_eq!(fm.get_field_marker(coord).and_then(|m| m.get_home_text()), Some("first"));

        let mut m2 = crate::marking::field_marker::FieldMarker::with_coordinate(coord);
        m2.set_home_text("second");
        fm.add_field_marker(m2);
        assert_eq!(fm.field_markers.len(), 1);
        assert_eq!(fm.get_field_marker(coord).and_then(|m| m.get_home_text()), Some("second"));
    }

    #[test]
    fn remove_field_marker_removes_entry() {
        let mut fm = FieldModel::new();
        let coord = FieldCoordinate::new(4, 4);
        fm.add_field_marker(crate::marking::field_marker::FieldMarker::with_coordinate(coord));
        assert!(fm.remove_field_marker(coord));
        assert!(fm.get_field_marker(coord).is_none());
        assert!(!fm.remove_field_marker(coord));
    }
}
