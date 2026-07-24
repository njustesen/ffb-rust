package com.fumbbl.ffb.client.state.logic.mixed;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.client.factory.LogicPluginFactory;
import com.fumbbl.ffb.client.state.logic.ClientAction;
import com.fumbbl.ffb.client.state.logic.plugin.BlockLogicExtensionPlugin;
import com.fumbbl.ffb.client.state.logic.plugin.LogicPlugin;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.Player;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.mockito.junit.jupiter.MockitoSettings;
import org.mockito.quality.Strictness;

import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertDoesNotThrow;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.when;

/**
 * Ported from {@code ffb-rust/crates/ffb-client/src/client/state/logic/mixed/block_kind_logic_module.rs}
 * against the real mixed {@link BlockKindLogicModule} (extends LogicModule; builds a
 * BlockLogicExtension → BLOCK plugin).
 *
 * <p>Rust tests pruned rather than ported (kept the suites 1:1):
 * {@code action_context_*} and {@code player_interaction_selects_action_with_game} route through
 * the real (unmockable) {@code BlockLogicExtension.blockActionContext} over a live Game; and
 * {@code player_interaction_ignores_without_game} / {@code perform_available_action_no_op_without_game}
 * are Rust no-game short-circuits.
 */
@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class BlockKindLogicModuleTest {

	@Mock
	FantasyFootballClient client;

	@Mock
	Game game;

	@Mock
	LogicPluginFactory logicPluginFactory;

	@Mock
	BlockLogicExtensionPlugin blockLogicExtensionPlugin;

	@SuppressWarnings("rawtypes")
	@Mock
	Player player;

	private BlockKindLogicModule module;

	@BeforeEach
	void setUp() {
		when(client.getGame()).thenReturn(game);
		when(game.<LogicPluginFactory>getFactory(FactoryType.Factory.LOGIC_PLUGIN)).thenReturn(logicPluginFactory);
		when(logicPluginFactory.forType(LogicPlugin.Type.BLOCK)).thenReturn(blockLogicExtensionPlugin);
		module = new BlockKindLogicModule(client);
	}

	// rust: available_actions_contains_expected_variants
	@Test
	void availableActionsContainsExpectedVariants() {
		Set<ClientAction> actions = module.availableActions();
		assertTrue(actions.contains(ClientAction.BLOCK));
		assertTrue(actions.contains(ClientAction.STAB));
		assertTrue(actions.contains(ClientAction.GORED_BY_THE_BULL));
		assertEquals(6, actions.size());
	}

	// rust: perform_available_action_skips_when_away_playing
	@Test
	void performAvailableActionSkipsWhenAwayPlaying() {
		when(game.isHomePlaying()).thenReturn(false);
		assertDoesNotThrow(() -> module.performAvailableAction(player, ClientAction.GORED_BY_THE_BULL));
	}
}
