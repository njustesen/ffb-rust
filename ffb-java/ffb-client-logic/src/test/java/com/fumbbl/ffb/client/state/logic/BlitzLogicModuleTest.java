package com.fumbbl.ffb.client.state.logic;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.client.factory.LogicPluginFactory;
import com.fumbbl.ffb.client.state.logic.plugin.BlockLogicExtensionPlugin;
import com.fumbbl.ffb.client.state.logic.plugin.LogicPlugin;
import com.fumbbl.ffb.client.state.logic.plugin.MoveLogicPlugin;
import com.fumbbl.ffb.model.ActingPlayer;
import com.fumbbl.ffb.model.FieldModel;
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
 * Ported from {@code ffb-rust/crates/ffb-client/src/client/state/logic/blitz_logic_module.rs}
 * against the real {@link BlitzLogicModule} (extends MoveLogicModule and builds a
 * BlockLogicExtension — construction needs both the MOVE and BLOCK plugins).
 *
 * <p>Rust tests pruned rather than ported (kept the suites 1:1):
 * {@code player_peek_resets_when_not_blockable} (routes through the unmockable
 * BlockLogicExtension.isBlockable over a live Game); and
 * {@code player_peek_resets_when_no_game} / {@code player_interaction_ignores_without_game} /
 * {@code perform_available_action_no_op_without_game} (Rust no-game short-circuits with no Java
 * counterpart).
 */
@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class BlitzLogicModuleTest {

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
	FieldModel fieldModel;

	@Mock
	ActingPlayer actingPlayer;

	private BlitzLogicModule module;

	@BeforeEach
	void setUp() {
		when(client.getGame()).thenReturn(game);
		when(game.<LogicPluginFactory>getFactory(FactoryType.Factory.LOGIC_PLUGIN)).thenReturn(logicPluginFactory);
		when(logicPluginFactory.forType(LogicPlugin.Type.MOVE)).thenReturn(moveLogicPlugin);
		when(logicPluginFactory.forType(LogicPlugin.Type.BLOCK)).thenReturn(blockLogicExtensionPlugin);
		when(game.getActingPlayer()).thenReturn(actingPlayer);
		when(game.getFieldModel()).thenReturn(fieldModel);
		module = new BlitzLogicModule(client);
	}

	// rust: available_actions_includes_extension_and_own_actions
	@Test
	void availableActionsIncludesExtensionAndOwnActions() {
		Set<ClientAction> actions = module.availableActions();
		assertTrue(actions.contains(ClientAction.MOVE));
		assertTrue(actions.contains(ClientAction.GORED_BY_THE_BULL));
		assertTrue(actions.contains(ClientAction.INCORPOREAL));
		assertTrue(actions.contains(ClientAction.BLOCK));
		assertTrue(actions.contains(ClientAction.TREACHEROUS));
	}

	// rust: move_action_is_blitz_move
	@Test
	void moveActionIsBlitzMove() {
		assertEquals(PlayerAction.BLITZ_MOVE, module.moveAction());
	}

	// rust: player_activation_used_falls_back_without_target_selection_state
	@Test
	void playerActivationUsedFallsBackWithoutTargetSelectionState() {
		when(fieldModel.getTargetSelectionState()).thenReturn(null);
		when(actingPlayer.hasActed()).thenReturn(false);
		assertFalse(module.playerActivationUsed());
	}

	// rust: is_gored_available_false_without_target_selection_state
	@Test
	void isGoredAvailableFalseWithoutTargetSelectionState() {
		when(fieldModel.getTargetSelectionState()).thenReturn(null);
		assertFalse(module.isGoredAvailable());
	}
}
