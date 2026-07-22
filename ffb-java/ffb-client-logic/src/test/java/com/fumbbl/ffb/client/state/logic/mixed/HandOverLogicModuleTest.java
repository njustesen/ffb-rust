package com.fumbbl.ffb.client.state.logic.mixed;

import com.fumbbl.ffb.ClientStateId;
import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.client.factory.LogicPluginFactory;
import com.fumbbl.ffb.client.state.logic.ClientAction;
import com.fumbbl.ffb.client.state.logic.interaction.InteractionResult;
import com.fumbbl.ffb.client.state.logic.plugin.LogicPlugin;
import com.fumbbl.ffb.client.state.logic.plugin.MoveLogicPlugin;
import com.fumbbl.ffb.model.ActingPlayer;
import com.fumbbl.ffb.model.FieldModel;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.Player;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.mockito.junit.jupiter.MockitoSettings;
import org.mockito.quality.Strictness;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.when;

/**
 * Ported from {@code ffb-rust/crates/ffb-client/src/client/state/logic/mixed/hand_over_logic_module.rs}
 * against the real mixed {@link HandOverLogicModule}.
 *
 * <p>Rust tests pruned rather than ported (kept the suites 1:1):
 * {@code action_context_always_adds_end_move} (Java {@code actionContext} fans out into ~12
 * availability helpers + jump mechanic + ball-in-hand — fixture-inexpressible), and
 * {@code ball_in_hand_false_without_game} / {@code player_interaction_ignores_without_game} /
 * {@code end_turn_no_op_without_game} (Rust {@code client.game()?} no-game short-circuits with no
 * Java counterpart).
 */
@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class HandOverLogicModuleTest {

	@Mock
	FantasyFootballClient client;

	@Mock
	Game game;

	@Mock
	LogicPluginFactory logicPluginFactory;

	@Mock
	MoveLogicPlugin moveLogicPlugin;

	@Mock
	ActingPlayer actingPlayer;

	@Mock
	FieldModel fieldModel;

	@SuppressWarnings("rawtypes")
	@Mock
	Player actor;

	@SuppressWarnings("rawtypes")
	@Mock
	Player catcher;

	private HandOverLogicModule module;

	@BeforeEach
	void setUp() {
		when(client.getGame()).thenReturn(game);
		when(game.<LogicPluginFactory>getFactory(FactoryType.Factory.LOGIC_PLUGIN)).thenReturn(logicPluginFactory);
		when(logicPluginFactory.forType(LogicPlugin.Type.MOVE)).thenReturn(moveLogicPlugin);
		when(game.getActingPlayer()).thenReturn(actingPlayer);
		when(game.getFieldModel()).thenReturn(fieldModel);
		when(actingPlayer.getPlayer()).thenReturn(actor);
		module = new HandOverLogicModule(client);
	}

	// rust: available_actions_delegates_to_move_logic
	@Test
	void availableActionsDelegatesToMoveLogic() {
		assertTrue(module.availableActions().contains(ClientAction.MOVE));
	}

	// rust: can_player_get_hand_over_false_without_catcher
	@Test
	void canPlayerGetHandOverFalseWithoutCatcher() {
		assertFalse(module.canPlayerGetHandOver(null));
	}

	// rust: can_player_get_hand_over_false_without_adjacency
	@Test
	void canPlayerGetHandOverFalseWithoutAdjacency() {
		when(fieldModel.getPlayerCoordinate(actor)).thenReturn(new FieldCoordinate(1, 1));
		when(fieldModel.getPlayerCoordinate(catcher)).thenReturn(new FieldCoordinate(10, 10));
		assertFalse(module.canPlayerGetHandOver(catcher));
	}

	// rust: field_peek_delegates_to_move_state
	@Test
	void fieldPeekDelegatesToMoveState() {
		InteractionResult result = module.fieldPeek(new FieldCoordinate(1, 1));
		assertEquals(InteractionResult.Kind.DELEGATE, result.getKind());
		assertEquals(ClientStateId.MOVE, result.getDelegate());
	}

	// rust: player_peek_ignores_when_not_eligible
	@Test
	void playerPeekIgnoresWhenNotEligible() {
		when(fieldModel.getPlayerCoordinate(actor)).thenReturn(new FieldCoordinate(1, 1));
		when(fieldModel.getPlayerCoordinate(catcher)).thenReturn(new FieldCoordinate(10, 10));
		InteractionResult result = module.playerPeek(catcher);
		assertEquals(InteractionResult.Kind.IGNORE, result.getKind());
	}
}
