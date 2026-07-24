package com.fumbbl.ffb.client.state.logic.mixed;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.client.factory.LogicPluginFactory;
import com.fumbbl.ffb.client.state.logic.ClientAction;
import com.fumbbl.ffb.client.state.logic.plugin.BlockLogicExtensionPlugin;
import com.fumbbl.ffb.client.state.logic.plugin.LogicPlugin;
import com.fumbbl.ffb.client.state.logic.plugin.MoveLogicPlugin;
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

/**
 * Ported from
 * {@code ffb-rust/crates/ffb-client/src/client/state/logic/mixed/putrid_regurgitation_blitz_logic_module.rs}
 * against the real mixed {@link PutridRegurgitationBlitzLogicModule} (extends BlitzLogicModule →
 * MOVE + BLOCK plugins).
 *
 * <p>Rust tests pruned rather than ported (kept the suites 1:1):
 * {@code player_peek_resets_without_game} / {@code player_interaction_ignores_without_game}
 * (Rust no-game short-circuits); {@code other_team_returns_away_for_home_id} (tests a Rust helper
 * mirroring {@code Game.getOtherTeam}, not a module method).
 */
@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class PutridRegurgitationBlitzLogicModuleTest {

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
	ActingPlayer actingPlayer;

	private PutridRegurgitationBlitzLogicModule module;

	@BeforeEach
	void setUp() {
		when(client.getGame()).thenReturn(game);
		when(game.<LogicPluginFactory>getFactory(FactoryType.Factory.LOGIC_PLUGIN)).thenReturn(logicPluginFactory);
		when(logicPluginFactory.forType(LogicPlugin.Type.MOVE)).thenReturn(moveLogicPlugin);
		when(logicPluginFactory.forType(LogicPlugin.Type.BLOCK)).thenReturn(blockLogicExtensionPlugin);
		when(game.getActingPlayer()).thenReturn(actingPlayer);
		module = new PutridRegurgitationBlitzLogicModule(client);
	}

	// rust: available_actions_has_expected_set
	@Test
	void availableActionsHasExpectedSet() {
		Set<ClientAction> actions = module.availableActions();
		assertTrue(actions.contains(ClientAction.PROJECTILE_VOMIT));
		assertTrue(actions.contains(ClientAction.MOVE));
		assertEquals(5, actions.size());
	}

	// rust: is_move_available_matches_own_action_only
	@Test
	void isMoveAvailableMatchesOwnActionOnly() {
		when(actingPlayer.getPlayerAction()).thenReturn(PlayerAction.PUTRID_REGURGITATION_BLITZ);
		assertTrue(module.isMoveAvailable(actingPlayer));
		when(actingPlayer.getPlayerAction()).thenReturn(PlayerAction.MOVE);
		assertFalse(module.isMoveAvailable(actingPlayer));
	}

	// rust: is_putrid_regurgitation_available_false_when_move_available
	@Test
	void isPutridRegurgitationAvailableFalseWhenMoveAvailable() {
		when(actingPlayer.getPlayerAction()).thenReturn(PlayerAction.PUTRID_REGURGITATION_BLITZ);
		assertFalse(module.isPutridRegurgitationAvailable());
	}

	// rust: is_putrid_regurgitation_available_false_without_block
	@Test
	void isPutridRegurgitationAvailableFalseWithoutBlock() {
		when(actingPlayer.getPlayerAction()).thenReturn(null);
		when(actingPlayer.hasBlocked()).thenReturn(false);
		assertFalse(module.isPutridRegurgitationAvailable());
	}
}
