package com.fumbbl.ffb.client.state.logic.bb2025;

import com.fumbbl.ffb.ClientStateId;
import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.client.factory.LogicPluginFactory;
import com.fumbbl.ffb.client.state.logic.ClientAction;
import com.fumbbl.ffb.client.state.logic.plugin.BlockLogicExtensionPlugin;
import com.fumbbl.ffb.client.state.logic.plugin.LogicPlugin;
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
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.when;

// Mirrors ffb-rust ffb-client crates/ffb-client/src/client/state/logic/bb2025/synchronous_multi_block_logic_module.rs
// (Rust: mod tests). SynchronousMultiBlockLogicModule extends LogicModule directly but also
// builds a BlockLogicExtension field, whose constructor eagerly resolves a
// BlockLogicExtensionPlugin via client.getGame().getFactory(...).forType(LogicPlugin.Type.BLOCK);
// game/factory are mocked explicitly (not deep-stub cascaded) and wired to a real plugin mock so
// construction succeeds.
//
// SKIPPED (with reasons):
// - set_up_clears_selection_state / select_player_stores_block_kind_and_state /
//   select_player_ignored_once_two_are_selected / handle_player_selection_deselects_when_already_selected /
//   perform_available_action_end_move_clears_selection: these all assert on the private
//   `selectedPlayers`/`originalPlayerStates` fields or call the private `selectPlayer(Player)`
//   method directly — not accessible from a test in Java (Rust exposes them as `pub(crate)`-visible
//   struct fields for its `RefCell` workaround, but the Java originals are `private`).
// - action_context_adds_move_when_suffering_blood_lust: `actionContext(ActingPlayer)` fans out
//   into isXAvailable(actingPlayer) helpers reading a live Game — out of scope.
// - player_peek_resets_when_not_blockable / player_interaction_ignores_without_game: require a
//   live Game/Player graph via BlockLogicExtension.isBlockable(...) — out of scope.
@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class SynchronousMultiBlockLogicModuleTest {

	@Mock
	FantasyFootballClient client;

	@Mock
	Game game;

	@Mock
	LogicPluginFactory logicPluginFactory;

	@Mock
	BlockLogicExtensionPlugin blockLogicExtensionPlugin;

	@BeforeEach
	void setUp() {
		when(client.getGame()).thenReturn(game);
		when(game.<LogicPluginFactory>getFactory(FactoryType.Factory.LOGIC_PLUGIN)).thenReturn(logicPluginFactory);
		when(logicPluginFactory.forType(LogicPlugin.Type.BLOCK)).thenReturn(blockLogicExtensionPlugin);
	}

	@Test
	void getIdReturnsSynchronousMultiBlock() {
		SynchronousMultiBlockLogicModule module = new SynchronousMultiBlockLogicModule(client);
		assertEquals(ClientStateId.SYNCHRONOUS_MULTI_BLOCK, module.getId());
	}

	@Test
	void availableActionsContainsBlockAndMove() {
		SynchronousMultiBlockLogicModule module = new SynchronousMultiBlockLogicModule(client);
		Set<ClientAction> actions = module.availableActions();
		assertTrue(actions.contains(ClientAction.BLOCK));
		assertTrue(actions.contains(ClientAction.MOVE));
		assertEquals(11, actions.size());
	}
}
