package com.fumbbl.ffb.client.state.logic.mixed;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.client.factory.LogicPluginFactory;
import com.fumbbl.ffb.client.net.ClientCommunication;
import com.fumbbl.ffb.client.state.logic.ClientAction;
import com.fumbbl.ffb.client.state.logic.plugin.BlockLogicExtensionPlugin;
import com.fumbbl.ffb.client.state.logic.plugin.LogicPlugin;
import com.fumbbl.ffb.model.ActingPlayer;
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
 * Ported from
 * {@code ffb-rust/crates/ffb-client/src/client/state/logic/mixed/putrid_regurgitation_block_logic_module.rs}
 * against the real mixed {@link PutridRegurgitationBlockLogicModule} (extends BlockLogicModule →
 * BLOCK plugin).
 *
 * <p>Rust tests pruned rather than ported (kept the suites 1:1):
 * {@code player_peek_resets_without_game} / {@code player_interaction_ignores_without_game}
 * (Rust no-game short-circuits).
 */
@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class PutridRegurgitationBlockLogicModuleTest {

	@Mock
	FantasyFootballClient client;

	@Mock
	Game game;

	@Mock
	LogicPluginFactory logicPluginFactory;

	@Mock
	BlockLogicExtensionPlugin blockLogicExtensionPlugin;

	@Mock
	ActingPlayer actingPlayer;

	@Mock
	ClientCommunication communication;

	@SuppressWarnings("rawtypes")
	@Mock
	Player player;

	private PutridRegurgitationBlockLogicModule module;

	@BeforeEach
	void setUp() {
		when(client.getGame()).thenReturn(game);
		when(game.<LogicPluginFactory>getFactory(FactoryType.Factory.LOGIC_PLUGIN)).thenReturn(logicPluginFactory);
		when(logicPluginFactory.forType(LogicPlugin.Type.BLOCK)).thenReturn(blockLogicExtensionPlugin);
		when(game.getActingPlayer()).thenReturn(actingPlayer);
		when(client.getCommunication()).thenReturn(communication);
		module = new PutridRegurgitationBlockLogicModule(client);
	}

	// rust: available_actions_has_expected_set
	@Test
	void availableActionsHasExpectedSet() {
		Set<ClientAction> actions = module.availableActions();
		assertEquals(2, actions.size());
		assertTrue(actions.contains(ClientAction.PROJECTILE_VOMIT));
		assertTrue(actions.contains(ClientAction.END_MOVE));
	}

	// rust: perform_available_action_no_op_for_unknown_action
	@Test
	void performAvailableActionNoOpForUnknownAction() {
		assertDoesNotThrow(() -> module.performAvailableAction(player, ClientAction.MOVE));
	}
}
