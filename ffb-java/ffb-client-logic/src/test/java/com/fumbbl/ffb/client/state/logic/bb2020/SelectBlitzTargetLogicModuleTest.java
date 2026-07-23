package com.fumbbl.ffb.client.state.logic.bb2020;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.client.factory.LogicPluginFactory;
import com.fumbbl.ffb.client.net.ClientCommunication;
import com.fumbbl.ffb.client.state.logic.ClientAction;
import com.fumbbl.ffb.client.state.logic.plugin.BlockLogicExtensionPlugin;
import com.fumbbl.ffb.client.state.logic.plugin.LogicPlugin;
import com.fumbbl.ffb.client.state.logic.plugin.MoveLogicPlugin;
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

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

/**
 * Ported from
 * {@code ffb-rust/crates/ffb-client/src/client/state/logic/bb2020/select_blitz_target_logic_module.rs}
 * against the real bb2020 {@link SelectBlitzTargetLogicModule} (extends MoveLogicModule and builds
 * a BlockLogicExtension — construction needs both the MOVE and BLOCK plugins).
 *
 * <p>Rust tests pruned rather than ported (kept the suites 1:1):
 * {@code action_context_always_has_end_move} (actionContext fans out into isXAvailable helpers
 * needing the GAME-mechanic factory chain — NPEs with targeted mocks);
 * {@code player_peek_invalid_without_game} / {@code player_peek_invalid_when_not_valid_blitz_target}
 * (playerPeek touches {@code client.getUserInterface()} / the unmockable BlockLogicExtension over a
 * live Game); {@code player_interaction_ignores_without_game} (Rust no-game short-circuit).
 */
@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class SelectBlitzTargetLogicModuleTest {

	@Mock
	FantasyFootballClient client;

	@Mock
	Game game;

	@Mock
	LogicPluginFactory logicPluginFactory;

	@Mock
	MoveLogicPlugin moveLogicPlugin;

	@Mock
	BlockLogicExtensionPlugin blockLogicExtensionPlugin;

	@Mock
	ClientCommunication communication;

	@SuppressWarnings("rawtypes")
	@Mock
	Player player;

	private SelectBlitzTargetLogicModule module;

	@BeforeEach
	void setUp() {
		when(client.getGame()).thenReturn(game);
		when(game.<LogicPluginFactory>getFactory(FactoryType.Factory.LOGIC_PLUGIN)).thenReturn(logicPluginFactory);
		when(logicPluginFactory.forType(LogicPlugin.Type.MOVE)).thenReturn(moveLogicPlugin);
		when(logicPluginFactory.forType(LogicPlugin.Type.BLOCK)).thenReturn(blockLogicExtensionPlugin);
		when(client.getCommunication()).thenReturn(communication);
		module = new SelectBlitzTargetLogicModule(client);
	}

	// rust: available_actions_matches_java
	@Test
	void availableActionsMatchesJava() {
		Set<ClientAction> actions = module.availableActions();
		assertTrue(actions.contains(ClientAction.END_MOVE));
		assertTrue(actions.contains(ClientAction.THEN_I_STARTED_BLASTIN));
		assertEquals(9, actions.size());
	}

	// rust: perform_available_action_end_move_sends_target_selected
	@Test
	void performAvailableActionEndMoveSendsTargetSelected() {
		when(player.getId()).thenReturn("p1");
		module.performAvailableAction(player, ClientAction.END_MOVE);
		verify(communication).sendTargetSelected("p1");
	}
}
