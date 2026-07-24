package com.fumbbl.ffb.client.state.logic.mixed;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.client.factory.LogicPluginFactory;
import com.fumbbl.ffb.client.state.logic.ClientAction;
import com.fumbbl.ffb.client.state.logic.interaction.InteractionResult;
import com.fumbbl.ffb.client.state.logic.plugin.BlockLogicExtensionPlugin;
import com.fumbbl.ffb.client.state.logic.plugin.LogicPlugin;
import com.fumbbl.ffb.model.ActingPlayer;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.model.Team;
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
 * Ported from {@code ffb-rust/crates/ffb-client/src/client/state/logic/mixed/maximum_carnage_logic_module.rs}
 * against the real mixed {@link MaximumCarnageLogicModule} (extends BlockLogicModule → BLOCK plugin;
 * its overridden playerPeek/availableActions avoid the extension).
 *
 * <p>Rust tests pruned rather than ported (kept the suites 1:1):
 * {@code player_peek_resets_without_game} / {@code player_interaction_ignores_without_game}
 * (Rust no-game short-circuits with no Java counterpart).
 */
@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class MaximumCarnageLogicModuleTest {

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
	Team actingTeam;

	@SuppressWarnings("rawtypes")
	@Mock
	Player actor;

	@SuppressWarnings("rawtypes")
	@Mock
	Player player;

	private MaximumCarnageLogicModule module;

	@BeforeEach
	void setUp() {
		when(client.getGame()).thenReturn(game);
		when(game.<LogicPluginFactory>getFactory(FactoryType.Factory.LOGIC_PLUGIN)).thenReturn(logicPluginFactory);
		when(logicPluginFactory.forType(LogicPlugin.Type.BLOCK)).thenReturn(blockLogicExtensionPlugin);
		when(game.getActingPlayer()).thenReturn(actingPlayer);
		when(game.getActingTeam()).thenReturn(actingTeam);
		when(actingPlayer.getPlayer()).thenReturn(actor);
		when(player.getId()).thenReturn("p1");
		when(game.getLastDefenderId()).thenReturn("other");
		module = new MaximumCarnageLogicModule(client);
	}

	// rust: available_actions_is_end_move_only
	@Test
	void availableActionsIsEndMoveOnly() {
		Set<ClientAction> actions = module.availableActions();
		assertTrue(actions.contains(ClientAction.END_MOVE));
		assertEquals(1, actions.size());
	}

	// rust: player_peek_perform_for_opponent_not_last_defender
	@Test
	void playerPeekPerformForOpponentNotLastDefender() {
		when(actingTeam.hasPlayer(player)).thenReturn(false);
		InteractionResult result = module.playerPeek(player);
		assertEquals(InteractionResult.Kind.PERFORM, result.getKind());
	}

	// rust: player_peek_resets_for_own_team_player
	@Test
	void playerPeekResetsForOwnTeamPlayer() {
		when(actingTeam.hasPlayer(player)).thenReturn(true);
		InteractionResult result = module.playerPeek(player);
		assertEquals(InteractionResult.Kind.RESET, result.getKind());
	}

	// rust: perform_available_action_no_op_for_unknown_action
	@Test
	void performAvailableActionNoOpForUnknownAction() {
		assertDoesNotThrow(() -> module.performAvailableAction(player, ClientAction.MOVE));
	}
}
