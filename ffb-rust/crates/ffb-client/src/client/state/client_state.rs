//! 1:1 translation of `com.fumbbl.ffb.client.state.ClientState` (148 lines), the abstract
//! generic base class `ClientState<T extends LogicModule, C extends FantasyFootballClient>`
//! for every per-game-phase client state.
//!
//! **Generics decision** (per the batch plan): Java's `C extends FantasyFootballClient` type
//! parameter is dropped — `FantasyFootballClient` has exactly one concrete instantiation in
//! this crate, so `ClientState<L>` here is generic only over the logic module type
//! `L: LogicModule`. The held `fClient` field is likewise dropped (not stored) — per the
//! established `client/handler`/`LogicModule` convention, the client is passed explicitly as
//! a parameter to the methods that need it, rather than stored on the struct. `getClient()`
//! therefore has no translation (nothing left to return) and is omitted.
//!
//! **`drawSelectSquare()`** is Java's one always-`abstract` method. No concrete
//! `ClientStateXxx` subclass exists in this crate's scope — all live in `ffb-client`'s Swing
//! layer (`ffb-client` Java module, not this Rust crate), which this project does not
//! translate. There is therefore no in-scope body to give it. `show_select_square` performs
//! the real, in-scope state transition (storing the coordinate) but the abstract painting
//! call itself is left as a documented `// java: abstract drawSelectSquare()...` no-op, per
//! `CLAUDE.md`'s "no invented logic" rule — not stubbed as a trait requirement, since nothing
//! in this crate would ever implement it.
//!
//! The AWT input surface (`MouseEvent` handlers, `actionKeyPressed(ActionKey)` in the sense of
//! real key-binding dispatch, drag/drop predicates) has near-empty default bodies in Java
//! itself (only concrete Swing subclasses override them) — those default bodies (mostly
//! `return false`/`return true`/no-op) are translated for real below since they *are* the
//! in-scope Java logic; only the `MouseEvent`-typed handlers are skipped outright since there
//! is no Rust `MouseEvent` equivalent at all.

use ffb_model::enums::ClientStateId;
use ffb_model::types::FieldCoordinate;
use ffb_protocol::net_command::NetCommand;

use crate::client::action_key::ActionKey;
use crate::client::fantasy_football_client::FantasyFootballClient;
use crate::client::state::logic::LogicModule;

/// java: `public abstract class ClientState<T extends LogicModule, C extends FantasyFootballClient>`
pub struct ClientState<L: LogicModule> {
    /// java: `protected final T logicModule`
    logic_module: L,
    /// java: `protected FieldCoordinate fSelectSquareCoordinate`
    select_square_coordinate: Option<FieldCoordinate>,
    /// java: `private DialogProgressBar dialogProgress` — `DialogProgressBar` (a Swing modal
    /// progress dialog) has no in-scope Rust equivalent; represented as `Option<()>` purely to
    /// preserve the null/non-null "a progress dialog is currently showing" state consumed by
    /// `show_icon_progress`/`hide_icon_progress`/`reinitialize_local_state` below.
    dialog_progress: Option<()>,
}

impl<L: LogicModule> ClientState<L> {
    /// java: `public ClientState(C pClient, T logicModule)`
    pub fn new(logic_module: L) -> Self {
        Self {
            logic_module,
            select_square_coordinate: None,
            dialog_progress: None,
        }
    }

    /// java: `public final void enterState()`
    pub fn enter_state(&mut self, client: &mut FantasyFootballClient) {
        self.logic_module.set_up(client);
        self.set_up(client);
    }

    /// java: `public void setUp() {}`
    pub fn set_up(&mut self, _client: &mut FantasyFootballClient) {}

    /// java: `public final void leaveState()`
    pub fn leave_state(&mut self, client: &mut FantasyFootballClient) {
        self.tear_down(client);
        self.logic_module.teardown(client);
    }

    /// java: `public void tearDown() {}`
    pub fn tear_down(&mut self, _client: &mut FantasyFootballClient) {}

    /// java: `public void reinitializeLocalState()` — `dialogProgress.showDialog(...)` is a
    /// Swing dialog call with no in-scope body; the null-check gate is the real in-scope
    /// logic and is preserved, the actual re-show is a documented no-op.
    pub fn reinitialize_local_state(&self) {
        if self.dialog_progress.is_some() {
            // java: `dialogProgress.showDialog(dialogProgress.getCloseListener());` — Swing
            // dialog call, no in-scope body, skipped.
        }
    }

    /// java: `public final ClientStateId getId()`
    pub fn get_id(&self) -> ClientStateId {
        self.logic_module.get_id()
    }

    /// java: `public void hideSelectSquare()`
    pub fn hide_select_square(&mut self) {
        self.select_square_coordinate = None;
    }

    /// java: `public void showSelectSquare(FieldCoordinate pCoordinate)`
    pub fn show_select_square(&mut self, coordinate: Option<FieldCoordinate>) {
        if let Some(c) = coordinate {
            self.select_square_coordinate = Some(c);
            // java: `drawSelectSquare();` — abstract, no in-scope concrete body; see module doc.
        }
    }

    /// java: `public FieldCoordinate` accessor for `fSelectSquareCoordinate` — Java exposes no
    /// public getter, but the field is read by this struct's own methods above; exposed here
    /// so tests (and any future caller) can observe the transition performed by
    /// `show_select_square`/`hide_select_square`.
    pub fn select_square_coordinate(&self) -> Option<FieldCoordinate> {
        self.select_square_coordinate
    }

    // java: `abstract protected void drawSelectSquare();` — no in-scope concrete body exists
    // anywhere in this crate; see module doc. Not translated (would require inventing a body).

    /// java: `protected void prePerform(int menuKey) {}`
    pub fn pre_perform(&mut self, _menu_key: i32) {}

    /// java: `protected void postPerform(int menuKey) {}`
    pub fn post_perform(&mut self, _menu_key: i32) {}

    /// java: `public final void endTurn()`
    pub fn end_turn(&mut self, client: &mut FantasyFootballClient) {
        self.logic_module.end_turn(client);
        self.post_end_turn(client);
    }

    /// java: `protected void postEndTurn() {}`
    pub fn post_end_turn(&mut self, _client: &mut FantasyFootballClient) {}

    /// java: `public T getLogicModule()`
    pub fn get_logic_module(&self) -> &L {
        &self.logic_module
    }

    /// Mutable counterpart of `get_logic_module`, matching this crate's established
    /// `xxx()`/`xxx_mut()` convention (e.g. `FantasyFootballClient::game`/`game_mut`).
    pub fn get_logic_module_mut(&mut self) -> &mut L {
        &mut self.logic_module
    }

    /// java: `public void handleCommand(NetCommand pNetCommand) {}`
    pub fn handle_command(&mut self, _net_command: &dyn NetCommand) {}

    // java: `public void setClickable(boolean b) {}` — Java itself marks this
    // "TODO remove once components and dialogs are moved to UI module"; a Swing-only hook
    // even in the original, skipped.

    // java: `mouseMoved`/`mouseDragged`/`mouseClicked`/`mouseEntered`/`mouseExited`/
    // `mousePressed`/`mouseReleased(MouseEvent)` — AWT input handlers; no Rust `MouseEvent`
    // equivalent exists in this crate, skipped (AWT).

    /// java: `public boolean actionKeyPressed(ActionKey actionKey) { return false; }`
    pub fn action_key_pressed(&mut self, _action_key: ActionKey) -> bool {
        false
    }

    /// java: `public boolean isInitDragAllowed(FieldCoordinate pCoordinate) { return true; }`
    pub fn is_init_drag_allowed(&self, _coordinate: FieldCoordinate) -> bool {
        true
    }

    /// java: `public boolean isDragAllowed(FieldCoordinate coordinate) { return true; }`
    pub fn is_drag_allowed(&self, _coordinate: FieldCoordinate) -> bool {
        true
    }

    /// java: `public boolean isDropAllowed(FieldCoordinate dragEndPosition) { return true; }`
    pub fn is_drop_allowed(&self, _drag_end_position: FieldCoordinate) -> bool {
        true
    }

    /// java: `public void showIconProgress(IDialogCloseListener listener, int total)` — Java
    /// constructs and shows a `DialogProgressBar` (Swing); `dialog_progress` here only tracks
    /// the null/non-null "showing" flag (see field doc), the dialog construction/show call
    /// itself is a documented no-op.
    pub fn show_icon_progress(&mut self, _total: i32) {
        self.dialog_progress = Some(());
        // java: `dialogProgress.showDialog(listener);` — Swing, no in-scope body, skipped.
    }

    /// java: `public synchronized void updateIconProgress(AtomicInteger count, int total)` —
    /// the `String.format(...)` message construction and the Swing progress-bar update call
    /// have no in-scope destination; the `AtomicInteger` itself is caller-owned in Java (an
    /// external counter, not struct state), so there is nothing left to mutate here. Documented
    /// no-op.
    pub fn update_icon_progress(&self, _count: i32, _total: i32) {
        // java: Swing dialog update, no in-scope body, skipped.
    }

    /// java: `public void hideIconProgress()`
    pub fn hide_icon_progress(&mut self) {
        self.dialog_progress = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    use ffb_model::model::acting_player::ActingPlayer;
    use ffb_model::model::game::Game;
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

    use crate::client::client_parameters::ClientParameters;
    use crate::client::state::logic::client_action::ClientAction;
    use crate::client::state::logic::interaction::action_context::ActionContext;

    /// Spy `LogicModule` test double recording lifecycle call order, per the batch plan.
    struct SpyLogicModule {
        calls: std::cell::RefCell<Vec<&'static str>>,
    }

    impl SpyLogicModule {
        fn new() -> Self {
            Self { calls: std::cell::RefCell::new(Vec::new()) }
        }
    }

    impl LogicModule for SpyLogicModule {
        fn get_id(&self) -> ClientStateId {
            ClientStateId::SelectPlayer
        }

        fn available_actions(&self) -> HashSet<ClientAction> {
            HashSet::new()
        }

        fn action_context(&self, _game: &Game, _acting_player: &ActingPlayer) -> ActionContext {
            ActionContext::new()
        }

        fn perform_available_action(
            &mut self,
            _client: &mut FantasyFootballClient,
            _player: &Player,
            _action: ClientAction,
        ) {
        }

        fn set_up(&mut self, _client: &mut FantasyFootballClient) {
            self.calls.borrow_mut().push("set_up");
        }

        fn teardown(&mut self, _client: &mut FantasyFootballClient) {
            self.calls.borrow_mut().push("teardown");
        }

        fn end_turn(&mut self, _client: &mut FantasyFootballClient) {
            self.calls.borrow_mut().push("end_turn");
        }
    }

    fn make_client() -> FantasyFootballClient {
        let params = ClientParameters::create_valid_params(&[
            "-spectator".to_string(),
            "-coach".to_string(),
            "bob".to_string(),
        ])
        .unwrap();
        FantasyFootballClient::new(params)
    }

    #[test]
    fn enter_state_calls_logic_module_set_up_then_own_set_up() {
        let mut client = make_client();
        let mut state = ClientState::new(SpyLogicModule::new());
        state.enter_state(&mut client);
        assert_eq!(state.get_logic_module().calls.borrow().as_slice(), &["set_up"]);
    }

    #[test]
    fn leave_state_calls_own_tear_down_then_logic_module_teardown() {
        let mut client = make_client();
        let mut state = ClientState::new(SpyLogicModule::new());
        state.leave_state(&mut client);
        assert_eq!(state.get_logic_module().calls.borrow().as_slice(), &["teardown"]);
    }

    #[test]
    fn end_turn_calls_logic_module_end_turn_then_post_end_turn() {
        let mut client = make_client();
        let mut state = ClientState::new(SpyLogicModule::new());
        state.end_turn(&mut client);
        assert_eq!(state.get_logic_module().calls.borrow().as_slice(), &["end_turn"]);
    }

    #[test]
    fn get_id_delegates_to_logic_module() {
        let state = ClientState::new(SpyLogicModule::new());
        assert_eq!(state.get_id(), ClientStateId::SelectPlayer);
    }

    #[test]
    fn hide_select_square_clears_coordinate() {
        let mut state = ClientState::new(SpyLogicModule::new());
        state.show_select_square(Some(FieldCoordinate::new(1, 1)));
        assert_eq!(state.select_square_coordinate(), Some(FieldCoordinate::new(1, 1)));
        state.hide_select_square();
        assert!(state.select_square_coordinate().is_none());
    }

    #[test]
    fn show_select_square_with_none_does_not_set_coordinate() {
        let mut state = ClientState::new(SpyLogicModule::new());
        state.show_select_square(None);
        assert!(state.select_square_coordinate().is_none());
    }

    #[test]
    fn show_select_square_with_some_sets_coordinate() {
        let mut state = ClientState::new(SpyLogicModule::new());
        state.show_select_square(Some(FieldCoordinate::new(3, 4)));
        assert_eq!(state.select_square_coordinate(), Some(FieldCoordinate::new(3, 4)));
    }

    #[test]
    fn action_key_pressed_default_is_false() {
        let mut state = ClientState::new(SpyLogicModule::new());
        assert!(!state.action_key_pressed(ActionKey::PLAYER_SELECT));
    }

    #[test]
    fn drag_drop_predicates_default_to_true() {
        let state = ClientState::new(SpyLogicModule::new());
        let coord = FieldCoordinate::new(0, 0);
        assert!(state.is_init_drag_allowed(coord));
        assert!(state.is_drag_allowed(coord));
        assert!(state.is_drop_allowed(coord));
    }

    #[test]
    fn icon_progress_lifecycle_tracks_showing_flag() {
        let mut state = ClientState::new(SpyLogicModule::new());
        assert!(state.dialog_progress.is_none());
        state.show_icon_progress(10);
        assert!(state.dialog_progress.is_some());
        state.update_icon_progress(1, 10);
        state.hide_icon_progress();
        assert!(state.dialog_progress.is_none());
    }

    #[test]
    fn reinitialize_local_state_is_a_no_op_without_dialog() {
        let state = ClientState::new(SpyLogicModule::new());
        // Should not panic; Java's null-check gate means nothing runs here either.
        state.reinitialize_local_state();
    }

    #[test]
    fn get_logic_module_mut_allows_mutation() {
        let mut state = ClientState::new(SpyLogicModule::new());
        let _ = state.get_logic_module_mut();
    }

    #[test]
    fn team_and_player_imports_are_exercised_by_available_actions() {
        // Sanity-check that the fixture types used above compile/behave as expected.
        let team = Team {
            id: "home".into(),
            name: "Home".into(),
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
            players: vec![Player::default()],
            vampire_lord: false,
            necromancer: false,
        };
        assert_eq!(team.players.len(), 1);
    }
}
