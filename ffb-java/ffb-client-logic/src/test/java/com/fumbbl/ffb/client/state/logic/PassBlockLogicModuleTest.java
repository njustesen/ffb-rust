package com.fumbbl.ffb.client.state.logic;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.TurnMode;
import com.fumbbl.ffb.client.ClientData;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.client.factory.LogicPluginFactory;
import com.fumbbl.ffb.client.net.ClientCommunication;
import com.fumbbl.ffb.client.state.logic.interaction.ActionContext;
import com.fumbbl.ffb.client.state.logic.interaction.InteractionResult;
import com.fumbbl.ffb.client.state.logic.plugin.LogicPlugin;
import com.fumbbl.ffb.client.state.logic.plugin.MoveLogicPlugin;
import com.fumbbl.ffb.factory.INamedObjectFactory;
import com.fumbbl.ffb.mechanics.Mechanic;
import com.fumbbl.ffb.mechanics.OnTheBallMechanic;
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

import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

/**
 * Ported from {@code ffb-rust/crates/ffb-client/src/client/state/logic/pass_block_logic_module.rs}
 * against the real {@link PassBlockLogicModule} (extends MoveLogicModule). All Rust tests port.
 */
@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class PassBlockLogicModuleTest {

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

	@Mock
	ClientCommunication communication;

	@Mock
	ClientData clientData;

	@Mock
	PlayerState playerState;

	@Mock
	OnTheBallMechanic onTheBallMechanic;

	@SuppressWarnings("rawtypes")
	@Mock
	INamedObjectFactory mechanicFactory;

	@SuppressWarnings("rawtypes")
	@Mock
	Player player;

	private PassBlockLogicModule module;

	@BeforeEach
	void setUp() {
		when(client.getGame()).thenReturn(game);
		when(game.<LogicPluginFactory>getFactory(FactoryType.Factory.LOGIC_PLUGIN)).thenReturn(logicPluginFactory);
		when(logicPluginFactory.forType(LogicPlugin.Type.MOVE)).thenReturn(moveLogicPlugin);
		when(game.getActingPlayer()).thenReturn(actingPlayer);
		when(game.getFieldModel()).thenReturn(fieldModel);
		when(client.getCommunication()).thenReturn(communication);
		when(client.getClientData()).thenReturn(clientData);
		module = new PassBlockLogicModule(client);
	}

	// rust: available_actions_has_four_entries
	@Test
	void availableActionsHasFourEntries() {
		Set<ClientAction> actions = module.availableActions();
		assertEquals(4, actions.size());
		assertTrue(actions.contains(ClientAction.MOVE));
	}

	// rust: action_context_for_acting_player_is_unsupported
	@Test
	void actionContextForActingPlayerIsUnsupported() {
		assertThrows(UnsupportedOperationException.class, () -> module.actionContext(actingPlayer));
	}

	// rust: action_context_for_player_adds_move_without_acting_player
	@Test
	void actionContextForPlayerAddsMoveWithoutActingPlayer() {
		when(actingPlayer.getPlayer()).thenReturn(null);
		when(fieldModel.getPlayerState(player)).thenReturn(playerState);
		when(playerState.isAbleToMove()).thenReturn(true);
		ActionContext actionContext = module.actionContext(player);
		assertTrue(actionContext.getActions().contains(ClientAction.MOVE));
	}

	// rust: is_turn_ending_false_without_acting_player
	@SuppressWarnings({"rawtypes", "unchecked"})
	@Test
	void isTurnEndingFalseWithoutActingPlayer() {
		when(actingPlayer.getPlayer()).thenReturn(null);
		when(game.<INamedObjectFactory>getFactory(FactoryType.Factory.MECHANIC)).thenReturn(mechanicFactory);
		when(mechanicFactory.forName(Mechanic.Type.ON_THE_BALL.name())).thenReturn(onTheBallMechanic);
		when(onTheBallMechanic.hasReachedValidPosition(any(), any())).thenReturn(false);
		assertFalse(module.isTurnEnding());
	}

	// rust: field_interaction_ignores_without_move_square
	@Test
	void fieldInteractionIgnoresWithoutMoveSquare() {
		when(fieldModel.getMoveSquare(new FieldCoordinate(3, 3))).thenReturn(null);
		InteractionResult result = module.fieldInteraction(new FieldCoordinate(3, 3));
		assertEquals(InteractionResult.Kind.IGNORE, result.getKind());
	}

	// rust: end_turn_sends_when_no_acting_player
	@Test
	void endTurnSendsWhenNoActingPlayer() {
		when(actingPlayer.getPlayer()).thenReturn(null);
		when(game.getTurnMode()).thenReturn(TurnMode.PASS_BLOCK);
		module.endTurn();
		verify(communication).sendEndTurn(TurnMode.PASS_BLOCK);
		verify(clientData).setEndTurnButtonHidden(true);
	}

	// rust: perform_available_action_move_sends_command
	@Test
	void performAvailableActionMoveSendsCommand() {
		module.performAvailableAction(player, ClientAction.MOVE);
		verify(communication).sendActingPlayer(player, PlayerAction.MOVE, false);
	}
}
