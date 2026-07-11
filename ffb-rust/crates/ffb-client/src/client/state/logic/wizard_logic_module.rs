//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.WizardLogicModule`.
//!
//! Handles the `WIZARD` client state (the "Wizard" kickoff event's own turn mode, not a UI
//! wizard): while a wizard spell (`LIGHTNING`/`ZAP`/`FIREBALL`) is pending in `ClientData`, field
//! and player clicks/peeks are routed through target-validity checks before sending the spell
//! choice to the server. `spell_available` tracks whether the one-shot spell has already been
//! used this activation (Java's `spellAvailable` field).
//!
//! Documented gap: Java's `isValidZapTarget` checks `player instanceof RosterPlayer` — the
//! ported `Player` model does not carry a `RosterPlayer` subtype distinction (every `Player` in
//! this codebase already corresponds to what Java calls a `RosterPlayer`; there is no other
//! concrete `Player` subclass reachable via `FieldModel.getPlayer`), so this collapses to
//! `player.is_some()`.

use std::cell::Cell;
use std::collections::HashSet;

use ffb_model::enums::ClientStateId;
use ffb_model::model::acting_player::ActingPlayer;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::model::SpecialEffect;
use ffb_model::types::FieldCoordinate;

use crate::client::fantasy_football_client::FantasyFootballClient;
use crate::client::state::logic::client_action::ClientAction;
use crate::client::state::logic::interaction::action_context::ActionContext;
use crate::client::state::logic::interaction::interaction_result::InteractionResult;
use crate::client::state::logic::logic_module::LogicModule;

/// Java: `WizardLogicModule`.
///
/// `spell_available` is `Cell<bool>` rather than a plain `bool`: the `LogicModule` trait's
/// `field_interaction`/`player_interaction`/`field_peek`/`player_peek` default methods take
/// `&self` (see `logic_module.rs`), so an inherent method of the same name sharing a `&mut self`
/// receiver would resolve to the (wrong) trait default at an earlier auto-ref step during method
/// lookup. `Cell` gives this one-field, `Copy`-typed piece of state interior mutability so the
/// inherent overloads below can stay `&self`, matching the receiver `PlaceBallLogicModule`/
/// `PushbackLogicModule` already use for their own same-named inherent methods.
#[derive(Debug, Default)]
pub struct WizardLogicModule {
    /// Java: `private boolean spellAvailable`.
    spell_available: Cell<bool>,
}

impl WizardLogicModule {
    /// Java: `public WizardLogicModule(FantasyFootballClient pClient)`.
    pub fn new() -> Self {
        Self { spell_available: Cell::new(false) }
    }

    /// java: `public InteractionResult fieldPeek(FieldCoordinate pCoordinate)`
    pub fn field_peek(&self, client: &FantasyFootballClient, coordinate: FieldCoordinate) -> InteractionResult {
        match client.client_data().wizard_spell() {
            Some(wizard_spell) => self.determine_special_effect(coordinate, wizard_spell),
            None => InteractionResult::reset(),
        }
    }

    /// java: `public InteractionResult playerPeek(Player<?> pPlayer)`
    pub fn player_peek(&self, client: &FantasyFootballClient, player: &Player) -> InteractionResult {
        let wizard_spell = client.client_data().wizard_spell();
        let player_coordinate = client.game().and_then(|game| game.field_model.player_coordinate(&player.id));
        match (player_coordinate, wizard_spell) {
            (Some(coordinate), Some(wizard_spell)) => self.determine_special_effect(coordinate, wizard_spell),
            _ => InteractionResult::reset(),
        }
    }

    /// java: `private InteractionResult determineSpecialEffect(FieldCoordinate pCoordinate)`
    fn determine_special_effect(&self, coordinate: FieldCoordinate, wizard_spell: SpecialEffect) -> InteractionResult {
        if self.spell_available.get() {
            // java: `InteractionResult.perform().with(pCoordinate).with(wizardSpell)`
            InteractionResult::perform().with_coordinate(coordinate).with_special_effect(wizard_spell)
        } else {
            InteractionResult::ignore()
        }
    }

    /// java: `public InteractionResult fieldInteraction(FieldCoordinate pCoordinate)`
    pub fn field_interaction(
        &self,
        client: &mut FantasyFootballClient,
        coordinate: FieldCoordinate,
    ) -> InteractionResult {
        self.handle_click(client, Some(coordinate))
    }

    /// java: `public InteractionResult playerInteraction(Player<?> pPlayer)`
    pub fn player_interaction(&self, client: &mut FantasyFootballClient, player: &Player) -> InteractionResult {
        let player_coordinate = client.game().and_then(|game| game.field_model.player_coordinate(&player.id));
        self.handle_click(client, player_coordinate)
    }

    /// java: `private InteractionResult handleClick(FieldCoordinate pCoordinate)`
    fn handle_click(&self, client: &mut FantasyFootballClient, coordinate: Option<FieldCoordinate>) -> InteractionResult {
        let coordinate = match coordinate {
            Some(c) => c,
            None => return InteractionResult::ignore(),
        };
        let wizard_spell = match client.client_data().wizard_spell() {
            Some(effect) => effect,
            None => return InteractionResult::ignore(),
        };
        let valid = match wizard_spell {
            SpecialEffect::LIGHTNING => self.is_valid_lightning_target(client, coordinate),
            SpecialEffect::ZAP => self.is_valid_zap_target(client, coordinate),
            SpecialEffect::FIREBALL => self.is_valid_fireball_target(client, coordinate),
            _ => return InteractionResult::ignore(),
        };
        if valid {
            client.communication_mut().send_wizard_spell(wizard_spell, coordinate);
            self.spell_available.set(false);
            InteractionResult::handled()
        } else {
            InteractionResult::reset()
        }
    }

    /// java: `public boolean isValidLightningTarget(FieldCoordinate pCoordinate)`
    pub fn is_valid_lightning_target(&self, client: &FantasyFootballClient, coordinate: FieldCoordinate) -> bool {
        let game = match client.game() {
            Some(game) => game,
            None => return false,
        };
        is_valid_lightning_target(game, coordinate)
    }

    /// java: `public boolean isValidZapTarget(FieldCoordinate pCoordinate)`
    pub fn is_valid_zap_target(&self, client: &FantasyFootballClient, coordinate: FieldCoordinate) -> bool {
        let game = match client.game() {
            Some(game) => game,
            None => return false,
        };
        is_valid_zap_target(game, coordinate)
    }

    /// java: `public boolean isValidFireballTarget(FieldCoordinate pCoordinate)`
    pub fn is_valid_fireball_target(&self, client: &FantasyFootballClient, coordinate: FieldCoordinate) -> bool {
        let game = match client.game() {
            Some(game) => game,
            None => return false,
        };
        is_valid_fireball_target(game, coordinate)
    }
}

/// java: `WizardLogicModule.isValidLightningTarget(FieldCoordinate)`
fn is_valid_lightning_target(game: &Game, coordinate: FieldCoordinate) -> bool {
    let player_id = match game.field_model.player_at(coordinate) {
        Some(id) => id,
        None => return false,
    };
    if !game.team_away.has_player(player_id) {
        return false;
    }
    match game.field_model.player_state(player_id) {
        Some(state) => !state.is_stunned() && !state.is_prone(),
        None => false,
    }
}

/// java: `WizardLogicModule.isValidZapTarget(FieldCoordinate)` — see module doc for the
/// `RosterPlayer` documented gap.
fn is_valid_zap_target(game: &Game, coordinate: FieldCoordinate) -> bool {
    match game.field_model.player_at(coordinate) {
        Some(player_id) => game.team_away.has_player(player_id),
        None => false,
    }
}

/// java: `WizardLogicModule.isValidFireballTarget(FieldCoordinate)` — Java's
/// `findAdjacentCoordinates(pCoordinate, FieldCoordinateBounds.FIELD, 1, true)` includes the
/// center square itself (`includeCenter = true`); `FieldModel::adjacent_on_pitch` only returns
/// the surrounding ring, so the center coordinate is added back in explicitly when on the pitch.
fn is_valid_fireball_target(game: &Game, coordinate: FieldCoordinate) -> bool {
    let mut squares = game.field_model.adjacent_on_pitch(coordinate);
    if coordinate.is_on_pitch() {
        squares.push(coordinate);
    }
    squares.iter().any(|square| is_valid_lightning_target(game, *square))
}

impl LogicModule for WizardLogicModule {
    /// java: `public ClientStateId getId()`
    fn get_id(&self) -> ClientStateId {
        ClientStateId::Wizard
    }

    /// java: `public void setUp()`
    fn set_up(&mut self, _client: &mut FantasyFootballClient) {
        self.spell_available.set(true);
    }

    /// java: `public Set<ClientAction> availableActions()`
    fn available_actions(&self) -> HashSet<ClientAction> {
        HashSet::new()
    }

    /// java: `protected ActionContext actionContext(ActingPlayer actingPlayer)` — always throws
    /// `UnsupportedOperationException` in Java.
    fn action_context(&self, _game: &Game, _acting_player: &ActingPlayer) -> ActionContext {
        panic!("actionContext for acting player is not supported in wizard context");
    }

    /// java: `protected void performAvailableAction(Player<?> player, ClientAction action)` —
    /// empty body in Java.
    fn perform_available_action(
        &mut self,
        _client: &mut FantasyFootballClient,
        _player: &Player,
        _action: ClientAction,
    ) {
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::client_parameters::ClientParameters;
    use ffb_model::enums::{Rules, PS_PRONE, PS_STANDING};
    use ffb_model::model::player::Player as ModelPlayer;
    use ffb_model::model::player_state::PlayerState;
    use ffb_model::model::team::Team;

    fn make_team(id: &str) -> Team {
        Team {
            id: id.to_string(),
            name: id.to_string(),
            race: "human".into(),
            roster_id: "human".into(),
            coach: "coach".into(),
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
            special_rules: Vec::new(),
            players: Vec::new(),
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn make_client() -> FantasyFootballClient {
        let params =
            ClientParameters::create_valid_params(&["-spectator".into(), "-coach".into(), "bob".into()]).unwrap();
        let mut client = FantasyFootballClient::new(params);
        client.set_game(Game::new(make_team("home"), make_team("away"), Rules::Bb2025));
        client
    }

    fn add_away_player(client: &mut FantasyFootballClient, id: &str, coord: FieldCoordinate, base: u32) {
        let mut player = ModelPlayer::default();
        player.id = id.to_string();
        let game = client.game_mut().unwrap();
        game.team_away.players.push(player);
        game.field_model.set_player_coordinate(id, coord);
        game.field_model.set_player_state(id, PlayerState::new(base));
    }

    #[test]
    fn get_id_is_wizard() {
        let module = WizardLogicModule::new();
        assert_eq!(module.get_id(), ClientStateId::Wizard);
    }

    #[test]
    fn set_up_resets_spell_available() {
        let mut module = WizardLogicModule::new();
        let mut client = make_client();
        assert!(!module.spell_available.get());
        module.set_up(&mut client);
        assert!(module.spell_available.get());
    }

    #[test]
    fn is_valid_lightning_target_requires_standing_away_player() {
        let mut client = make_client();
        let coord = FieldCoordinate::new(5, 5);
        add_away_player(&mut client, "p1", coord, PS_STANDING);
        let module = WizardLogicModule::new();
        assert!(module.is_valid_lightning_target(&client, coord));

        client.game_mut().unwrap().field_model.set_player_state("p1", PlayerState::new(PS_PRONE));
        assert!(!module.is_valid_lightning_target(&client, coord));
    }

    #[test]
    fn is_valid_zap_target_requires_away_player_present() {
        let mut client = make_client();
        let coord = FieldCoordinate::new(6, 6);
        let module = WizardLogicModule::new();
        assert!(!module.is_valid_zap_target(&client, coord));
        add_away_player(&mut client, "p1", coord, PS_STANDING);
        assert!(module.is_valid_zap_target(&client, coord));
    }

    #[test]
    fn is_valid_fireball_target_checks_adjacent_and_center_squares() {
        let mut client = make_client();
        let center = FieldCoordinate::new(10, 10);
        let adjacent = center.add(1, 0);
        add_away_player(&mut client, "p1", adjacent, PS_STANDING);
        let module = WizardLogicModule::new();
        assert!(module.is_valid_fireball_target(&client, center));
    }

    #[test]
    fn handle_click_ignores_when_no_wizard_spell_pending() {
        let module = WizardLogicModule::new();
        let mut client = make_client();
        let result = module.field_interaction(&mut client, FieldCoordinate::new(1, 1));
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Ignore
        );
    }

    #[test]
    fn handle_click_sends_spell_and_clears_availability_on_valid_target() {
        let module = WizardLogicModule::new();
        module.spell_available.set(true);
        let mut client = make_client();
        let coord = FieldCoordinate::new(3, 3);
        add_away_player(&mut client, "p1", coord, PS_STANDING);
        client.client_data_mut().set_wizard_spell(Some(SpecialEffect::ZAP));
        let result = module.field_interaction(&mut client, coord);
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Handled
        );
        assert!(!module.spell_available.get());
    }

    #[test]
    fn handle_click_resets_on_invalid_target() {
        let module = WizardLogicModule::new();
        module.spell_available.set(true);
        let mut client = make_client();
        client.client_data_mut().set_wizard_spell(Some(SpecialEffect::ZAP));
        let result = module.field_interaction(&mut client, FieldCoordinate::new(3, 3));
        assert_eq!(
            result.get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Reset
        );
    }

    #[test]
    #[should_panic(expected = "actionContext for acting player is not supported in wizard context")]
    fn action_context_panics() {
        let module = WizardLogicModule::new();
        let client = make_client();
        let game = client.game().unwrap();
        let ap = ActingPlayer::new();
        module.action_context(game, &ap);
    }
}
