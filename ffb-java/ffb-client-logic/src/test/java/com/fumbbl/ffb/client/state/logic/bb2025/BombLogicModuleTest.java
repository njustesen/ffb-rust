package com.fumbbl.ffb.client.state.logic.bb2025;

import com.fumbbl.ffb.ClientStateId;
import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.TurnMode;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.client.state.logic.ClientAction;
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

// Mirrors ffb-rust ffb-client crates/ffb-client/src/client/state/logic/bb2025/bomb_logic_module.rs
// (Rust: mod tests), one test fn per Rust #[test] that is faithfully reproducible.
//
// SKIPPED (with reasons):
// - action_context_adds_end_move_by_default: actionContext(ActingPlayer) fans out into many
//   isXAvailable(actingPlayer) helpers on LogicModule that walk live Player/Game internals
//   (skills, properties); not safely mockable without risking wrong/guessed stubbing.
// - player_interaction_ignores_without_game / field_interaction_ignores_without_game /
//   field_peek_ignores_without_game / player_peek_sets_selected_player /
//   perform_available_action_no_op_without_game: these all require a real Game object graph
//   (or, for the Java version, none of them can express "no game" since client.getGame() is
//   called unconditionally and NPEs on a null return with lenient/deep-stub mocking) — out of
//   scope per task instructions.
@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class BombLogicModuleTest {

	@Mock(answer = Answers.RETURNS_DEEP_STUBS)
	FantasyFootballClient client;

	@Test
	void isEndTurnActionAvailableTrueWhenNotBombTurn() {
		when(client.getGame().getTurnMode()).thenReturn(TurnMode.REGULAR);
		when(client.getGame().getActingPlayer().isMustCompleteAction()).thenReturn(false);
		BombLogicModule module = new BombLogicModule(client);
		assertTrue(module.isEndTurnActionAvailable());
	}

	@Test
	void isEndTurnActionAvailableFalseWhenMustCompleteAction() {
		when(client.getGame().getTurnMode()).thenReturn(TurnMode.REGULAR);
		when(client.getGame().getActingPlayer().isMustCompleteAction()).thenReturn(true);
		BombLogicModule module = new BombLogicModule(client);
		assertFalse(module.isEndTurnActionAvailable());
	}

	@Test
	void playerIsAboutToThrowFalseByDefault() {
		when(client.getGame().getActingPlayer().getPlayerAction()).thenReturn(null);
		BombLogicModule module = new BombLogicModule(client);
		assertFalse(module.playerIsAboutToThrow());
	}

	@Test
	void playerIsAboutToThrowTrueForHailMaryBomb() {
		when(client.getGame().getActingPlayer().getPlayerAction()).thenReturn(PlayerAction.HAIL_MARY_BOMB);
		BombLogicModule module = new BombLogicModule(client);
		assertTrue(module.playerIsAboutToThrow());
	}

	@Test
	void showRangeRulerTrueByDefaultWithoutPassCoordinate() {
		when(client.getGame().getPassCoordinate()).thenReturn(null);
		BombLogicModule module = new BombLogicModule(client);
		assertTrue(module.showRangeRuler());
	}

	@Test
	void showRangeRulerFalseAfterDisabling() {
		BombLogicModule module = new BombLogicModule(client);
		module.setShowRangeRuler(false);
		assertFalse(module.showRangeRuler());
	}

	@Test
	void availableActionsContainsHailMaryBomb() {
		BombLogicModule module = new BombLogicModule(client);
		Set<ClientAction> actions = module.availableActions();
		assertTrue(actions.contains(ClientAction.HAIL_MARY_BOMB));
		assertTrue(actions.contains(ClientAction.END_MOVE));
	}
}
