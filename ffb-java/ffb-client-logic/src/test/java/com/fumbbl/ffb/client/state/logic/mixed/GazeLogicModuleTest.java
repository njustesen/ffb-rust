package com.fumbbl.ffb.client.state.logic.mixed;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.client.factory.LogicPluginFactory;
import com.fumbbl.ffb.client.state.logic.ClientAction;
import com.fumbbl.ffb.client.state.logic.interaction.InteractionResult;
import com.fumbbl.ffb.client.state.logic.plugin.LogicPlugin;
import com.fumbbl.ffb.client.state.logic.plugin.MoveLogicPlugin;
import com.fumbbl.ffb.model.Game;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.mockito.junit.jupiter.MockitoSettings;
import org.mockito.quality.Strictness;

import java.util.Collections;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.when;

/**
 * Ported from {@code ffb-rust/crates/ffb-client/src/client/state/logic/mixed/gaze_logic_module.rs}
 * against the real mixed {@link GazeLogicModule}.
 *
 * <p>Rust tests pruned rather than ported (kept the suites 1:1):
 * {@code can_be_gazed_false_without_adjacency} / {@code player_peek_invalid_when_not_gazeable}
 * (Java {@code canBeGazed} evaluates {@code UtilPlayer.canGaze} first — GAME-mechanic factory
 * chain + findOtherTeam + a gaze-capable actor — so the named condition can't be isolated with
 * targeted mocks); {@code player_activation_used_falls_back_without_target_selection_state}
 * (falls through to {@code super.playerActivationUsed()} over a live acting-player graph);
 * {@code perform_available_action_delegates_without_panicking} (a Rust smoke/no-panic test with
 * no assertion); and {@code player_interaction_ignores_without_game} /
 * {@code end_turn_no_op_without_game} (Rust no-game short-circuits with no Java counterpart).
 */
@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class GazeLogicModuleTest {

	@Mock
	FantasyFootballClient client;

	@Mock
	Game game;

	@Mock
	LogicPluginFactory logicPluginFactory;

	@Mock
	MoveLogicPlugin moveLogicPlugin;

	private GazeLogicModule module;

	@BeforeEach
	void setUp() {
		when(client.getGame()).thenReturn(game);
		when(game.<LogicPluginFactory>getFactory(FactoryType.Factory.LOGIC_PLUGIN)).thenReturn(logicPluginFactory);
		when(logicPluginFactory.forType(LogicPlugin.Type.MOVE)).thenReturn(moveLogicPlugin);
		module = new GazeLogicModule(client);
	}

	// rust: available_actions_delegates_to_move_logic
	@Test
	void availableActionsDelegatesToMoveLogic() {
		when(moveLogicPlugin.availableActions()).thenReturn(Collections.emptySet());
		assertTrue(module.availableActions().contains(ClientAction.MOVE));
	}

	// rust: can_be_gazed_false_without_victim (observed via playerPeek's null-victim guard → INVALID)
	@Test
	void canBeGazedFalseWithoutVictim() {
		InteractionResult result = module.playerPeek(null);
		assertEquals(InteractionResult.Kind.INVALID, result.getKind());
	}
}
