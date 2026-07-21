package com.fumbbl.ffb.client.state.logic.mixed;

import com.fumbbl.ffb.ClientStateId;
import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.TurnMode;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.client.state.logic.ClientAction;
import com.fumbbl.ffb.model.ActingPlayer;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Answers;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.mockito.junit.jupiter.MockitoSettings;
import org.mockito.quality.Strictness;

import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.when;

// Mirrors ffb-rust ffb-client crates/ffb-client/src/client/state/logic/mixed/bomb_logic_module.rs
// (Rust: mod tests), one test fn per Rust #[test] that is faithfully reproducible.
//
// SKIPPED (with reasons):
// - action_context_omits_end_move_when_must_complete_action / action_context_empty_without_special_availability:
//   Java's actionContext(ActingPlayer) reads client.getGame() internally and walks a long
//   chain of LogicModule availability helpers (isHailMaryPassActionAvailable,
//   isTreacherousAvailable, isWisdomAvailable, ...), each touching a real Game/Team/FieldModel
//   graph -- out of scope per the "actionContext() with a real ActingPlayer/Game object graph"
//   skip rule (same category as MoveLogicModuleTest's skipped actionContext test).
// - player_interaction_ignores_without_game / field_interaction_ignores_without_game /
//   player_peek_sets_selected_player / field_peek_ignores_without_range_ruler_enabled /
//   perform_available_action_no_op_for_unavailable_treacherous: all dereference
//   client.getGame() unconditionally and/or require FieldModel/coordinate graph state or the
//   PassMechanic factory lookup (generic-erasure cast, same category skipped in
//   DumpOffLogicModuleTest) -- out of scope.
@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class BombLogicModuleTest {

	@Mock(answer = Answers.RETURNS_DEEP_STUBS)
	FantasyFootballClient client;

	BombLogicModule module;

	@BeforeEach
	void setUp() {
		// BombLogicModule extends LogicModule directly with a trivial constructor (no
		// LOGIC_PLUGIN factory resolution), so a plain deep-stub client is enough.
		module = new BombLogicModule(client);
	}

	@Test
	void getIdReturnsBomb() {
		assertEquals(ClientStateId.BOMB, module.getId());
	}

	@Test
	void availableActionsHasExpectedSize() {
		Set<ClientAction> actions = module.availableActions();
		assertEquals(11, actions.size());
		assertTrue(actions.contains(ClientAction.HAIL_MARY_BOMB));
	}

	// Rust: show_range_ruler_defaults_true_without_pass_coordinate
	@Test
	void showRangeRulerDefaultsTrueWithoutPassCoordinate() {
		when(client.getGame().getPassCoordinate()).thenReturn(null);

		assertTrue(module.showRangeRuler());
	}

	// Rust: show_range_ruler_false_after_setting_false
	@Test
	void showRangeRulerFalseAfterSettingFalse() {
		module.setShowRangeRuler(false);

		assertFalse(module.showRangeRuler());
	}

	// Rust: is_end_turn_action_available_false_during_bomb_turn
	@Test
	void isEndTurnActionAvailableFalseDuringBombTurn() {
		when(client.getGame().getTurnMode()).thenReturn(TurnMode.BOMB_HOME);

		assertFalse(module.isEndTurnActionAvailable());
	}

	// Rust: is_end_turn_action_available_true_during_regular_turn
	@Test
	void isEndTurnActionAvailableTrueDuringRegularTurn() {
		when(client.getGame().getTurnMode()).thenReturn(TurnMode.REGULAR);

		assertTrue(module.isEndTurnActionAvailable());
	}

	// Rust: is_end_turn_action_available_false_when_must_complete_action
	@Test
	void isEndTurnActionAvailableFalseWhenMustCompleteAction() {
		when(client.getGame().getTurnMode()).thenReturn(TurnMode.REGULAR);
		ActingPlayer actingPlayer = new ActingPlayer(null);
		actingPlayer.setMustCompleteAction(true);
		when(client.getGame().getActingPlayer()).thenReturn(actingPlayer);

		assertFalse(module.isEndTurnActionAvailable());
	}

	// Rust: player_is_about_to_throw_true_for_throw_bomb_action
	@Test
	void playerIsAboutToThrowTrueForThrowBombAction() {
		ActingPlayer actingPlayer = new ActingPlayer(null);
		actingPlayer.setPlayerAction(PlayerAction.THROW_BOMB);
		when(client.getGame().getActingPlayer()).thenReturn(actingPlayer);

		assertTrue(module.playerIsAboutToThrow());
	}
}
