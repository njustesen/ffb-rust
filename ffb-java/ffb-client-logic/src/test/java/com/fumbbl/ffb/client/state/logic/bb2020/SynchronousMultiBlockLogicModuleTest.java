package com.fumbbl.ffb.client.state.logic.bb2020;

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

/**
 * Ported from
 * {@code ffb-rust/crates/ffb-client/src/client/state/logic/bb2020/synchronous_multi_block_logic_module.rs}
 * against the real bb2020 {@link SynchronousMultiBlockLogicModule} (builds a BlockLogicExtension,
 * whose ctor resolves a BlockLogicExtensionPlugin — wired to a mock).
 *
 * <p>Only {@code available_actions_matches_java} is faithfully reproducible here. The rest of the
 * Rust module's tests are pruned (kept the suites 1:1): {@code set_up_clears_selection_state} /
 * {@code handle_player_selection_unsets_already_selected} assert on / manipulate the
 * {@code selectedPlayers}/{@code originalPlayerStates} maps, which are {@code private} in Java
 * (Rust exposes them as crate-visible RefCell fields); {@code action_context_always_includes_end_move}
 * fans out into isXAvailable helpers over a live acting-player graph; and
 * {@code player_peek_resets_when_not_blockable} / {@code player_interaction_ignores_without_game} /
 * {@code perform_available_action_end_move_clears_selection} require BlockLogicExtension.isBlockable
 * over a live Game (the real extension can't be mocked) — fixture-inexpressible.
 */
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

	// rust: available_actions_matches_java
	@Test
	void availableActionsMatchesJava() {
		SynchronousMultiBlockLogicModule module = new SynchronousMultiBlockLogicModule(client);
		Set<ClientAction> actions = module.availableActions();
		assertTrue(actions.contains(ClientAction.BLOCK));
		assertTrue(actions.contains(ClientAction.STAB));
		assertEquals(12, actions.size());
	}
}
