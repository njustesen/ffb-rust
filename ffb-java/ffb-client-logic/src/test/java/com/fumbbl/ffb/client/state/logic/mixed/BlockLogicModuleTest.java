package com.fumbbl.ffb.client.state.logic.mixed;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.client.factory.LogicPluginFactory;
import com.fumbbl.ffb.client.net.ClientCommunication;
import com.fumbbl.ffb.client.state.logic.ClientAction;
import com.fumbbl.ffb.client.state.logic.interaction.InteractionResult;
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

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

/**
 * Ported from {@code ffb-rust/crates/ffb-client/src/client/state/logic/mixed/block_logic_module.rs}
 * against the real mixed {@link BlockLogicModule} (extends AbstractBlockLogicModule; its ctor also
 * builds a BlockLogicExtension, whose ctor resolves a BlockLogicExtensionPlugin — wired to a mock).
 *
 * <p>Rust tests pruned rather than ported (kept the suites 1:1):
 * {@code action_context_adds_end_move_when_has_acted} / {@code action_context_empty_without_acted_or_bloodlust}
 * (actionContext delegates to BlockLogicExtension.actionContext over a live skill/property graph —
 * fixture-inexpressible), and {@code player_peek_resets_without_game} /
 * {@code player_interaction_ignores_without_game} / {@code end_turn_no_op_without_game} (Rust
 * no-game short-circuits with no Java counterpart).
 */
@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class BlockLogicModuleTest {

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

	private BlockLogicModule module;

	@BeforeEach
	void setUp() {
		when(client.getGame()).thenReturn(game);
		when(game.<LogicPluginFactory>getFactory(FactoryType.Factory.LOGIC_PLUGIN)).thenReturn(logicPluginFactory);
		when(logicPluginFactory.forType(LogicPlugin.Type.BLOCK)).thenReturn(blockLogicExtensionPlugin);
		when(game.getActingPlayer()).thenReturn(actingPlayer);
		when(client.getCommunication()).thenReturn(communication);
		module = new BlockLogicModule(client);
	}

	// rust: available_actions_includes_move_and_extension
	@Test
	void availableActionsIncludesMoveAndExtension() {
		Set<ClientAction> actions = module.availableActions();
		assertTrue(actions.contains(ClientAction.MOVE));
		assertTrue(actions.contains(ClientAction.END_MOVE));
		assertTrue(actions.contains(ClientAction.BLOCK));
	}

	// rust: player_interaction_handles_blitz_action
	@Test
	void playerInteractionHandlesBlitzAction() {
		when(actingPlayer.getPlayer()).thenReturn(player);
		when(actingPlayer.getPlayerAction()).thenReturn(PlayerAction.BLITZ);
		when(actingPlayer.isSufferingBloodLust()).thenReturn(false);
		when(actingPlayer.isJumping()).thenReturn(false);
		InteractionResult result = module.playerInteraction(player);
		assertEquals(InteractionResult.Kind.HANDLED, result.getKind());
		verify(communication).sendActingPlayer(player, PlayerAction.BLITZ_MOVE, false);
	}

	// rust: perform_available_action_move_sends_command
	@Test
	void performAvailableActionMoveSendsCommand() {
		when(actingPlayer.isJumping()).thenReturn(false);
		module.performAvailableAction(player, ClientAction.MOVE);
		verify(communication).sendActingPlayer(player, PlayerAction.MOVE, false);
	}
}
