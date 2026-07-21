package com.fumbbl.ffb.client.state.logic.bb2025;

import com.fumbbl.ffb.ClientStateId;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.client.state.logic.ClientAction;
import com.fumbbl.ffb.model.ActingPlayer;
import com.fumbbl.ffb.model.Game;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.mockito.junit.jupiter.MockitoSettings;
import org.mockito.quality.Strictness;

import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.when;

// Mirrors ffb-rust ffb-client crates/ffb-client/src/client/state/logic/bb2025/throw_keg_logic_module.rs
// (Rust: mod tests). ThrowKegLogicModule extends LogicModule directly (no plugin factory lookup),
// so a plain explicitly-wired Game/ActingPlayer mock suffices for construction.
//
// SKIPPED (with reasons):
// - is_valid_target_*: Java's `isValidTarget(Player, Game)` is `private`, not callable from a test
//   even in the same package.
// - action_context_adds_end_move_when_available: `actionContext(ActingPlayer)` fans out into many
//   isXAvailable(actingPlayer) helpers reading a live Game — out of scope.
// - set_up_and_teardown_do_not_panic_without_game / set_up_adds_move_squares_around_acting_player:
//   Java's `setUp()` calls `client.getGame().getFieldModel().findAdjacentCoordinates(...)` on the
//   real acting player's coordinate unconditionally (no null-game short circuit like Rust), so it
//   needs a live Game/FieldModel/Player graph to exercise safely — out of scope.
// - player_interaction_ignores_without_game: Java calls `client.getGame()` unconditionally and
//   would NPE rather than return ignore() for an absent game.
@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class ThrowKegLogicModuleTest {

	@Mock
	FantasyFootballClient client;

	@Mock
	Game game;

	@Mock
	ActingPlayer actingPlayer;

	@BeforeEach
	void setUp() {
		when(client.getGame()).thenReturn(game);
		when(game.getActingPlayer()).thenReturn(actingPlayer);
	}

	@Test
	void getIdReturnsThrowKeg() {
		ThrowKegLogicModule module = new ThrowKegLogicModule(client);
		assertEquals(ClientStateId.THROW_KEG, module.getId());
	}

	@Test
	void availableActionsContainsEndMoveAndWisdom() {
		ThrowKegLogicModule module = new ThrowKegLogicModule(client);
		Set<ClientAction> actions = module.availableActions();
		assertTrue(actions.contains(ClientAction.END_MOVE));
		assertTrue(actions.contains(ClientAction.WISDOM));
		assertEquals(9, actions.size());
	}

	@Test
	void isEndPlayerActionAvailableTrueWhenNotActed() {
		when(actingPlayer.hasActed()).thenReturn(false);
		ThrowKegLogicModule module = new ThrowKegLogicModule(client);
		assertTrue(module.isEndPlayerActionAvailable());
	}

	@Test
	void isEndPlayerActionAvailableFalseWhenActed() {
		when(actingPlayer.hasActed()).thenReturn(true);
		ThrowKegLogicModule module = new ThrowKegLogicModule(client);
		assertFalse(module.isEndPlayerActionAvailable());
	}
}
