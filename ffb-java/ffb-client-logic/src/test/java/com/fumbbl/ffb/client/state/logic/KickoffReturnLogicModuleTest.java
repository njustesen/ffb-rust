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
import com.fumbbl.ffb.model.ActingPlayer;
import com.fumbbl.ffb.model.FieldModel;
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

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

/**
 * Ported from {@code ffb-rust/crates/ffb-client/src/client/state/logic/kickoff_return_logic_module.rs}
 * against the real {@link KickoffReturnLogicModule} (extends MoveLogicModule). All Rust tests port.
 */
@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class KickoffReturnLogicModuleTest {

	@Mock
	FantasyFootballClient client;

	@Mock
	Game game;

	@Mock
	LogicPluginFactory logicPluginFactory;

	@Mock
	MoveLogicPlugin moveLogicPlugin;

	@Mock
	FieldModel fieldModel;

	@Mock
	ActingPlayer actingPlayer;

	@Mock
	Team teamHome;

	@Mock
	ClientCommunication communication;

	@Mock
	ClientData clientData;

	@Mock
	PlayerState playerState;

	@SuppressWarnings("rawtypes")
	@Mock
	Player player;

	private KickoffReturnLogicModule module;

	@BeforeEach
	void setUp() {
		when(client.getGame()).thenReturn(game);
		when(game.<LogicPluginFactory>getFactory(FactoryType.Factory.LOGIC_PLUGIN)).thenReturn(logicPluginFactory);
		when(logicPluginFactory.forType(LogicPlugin.Type.MOVE)).thenReturn(moveLogicPlugin);
		when(game.getFieldModel()).thenReturn(fieldModel);
		when(game.getActingPlayer()).thenReturn(actingPlayer);
		when(game.getTeamHome()).thenReturn(teamHome);
		when(client.getCommunication()).thenReturn(communication);
		when(client.getClientData()).thenReturn(clientData);
		module = new KickoffReturnLogicModule(client);
	}

	// rust: available_actions_is_move_and_end_move
	@Test
	void availableActionsIsMoveAndEndMove() {
		Set<ClientAction> actions = module.availableActions();
		assertTrue(actions.contains(ClientAction.MOVE));
		assertTrue(actions.contains(ClientAction.END_MOVE));
		assertEquals(2, actions.size());
	}

	// rust: action_context_for_acting_player_is_unsupported
	@Test
	void actionContextForActingPlayerIsUnsupported() {
		assertThrows(UnsupportedOperationException.class, () -> module.actionContext(actingPlayer));
	}

	// rust: action_context_for_player_adds_move_when_no_acting_player
	@Test
	void actionContextForPlayerAddsMoveWhenNoActingPlayer() {
		when(actingPlayer.getPlayer()).thenReturn(null);
		when(fieldModel.getPlayerState(player)).thenReturn(playerState);
		when(playerState.isAbleToMove()).thenReturn(true);
		ActionContext actionContext = module.actionContext(player);
		assertTrue(actionContext.getActions().contains(ClientAction.MOVE));
	}

	// rust: player_interaction_ignores_when_not_home_team
	@Test
	void playerInteractionIgnoresWhenNotHomeTeam() {
		when(teamHome.hasPlayer(player)).thenReturn(false);
		InteractionResult result = module.playerInteraction(player);
		assertEquals(InteractionResult.Kind.IGNORE, result.getKind());
	}

	// rust: field_interaction_ignores_without_move_square
	@Test
	void fieldInteractionIgnoresWithoutMoveSquare() {
		when(fieldModel.getMoveSquare(new FieldCoordinate(3, 3))).thenReturn(null);
		InteractionResult result = module.fieldInteraction(new FieldCoordinate(3, 3));
		assertEquals(InteractionResult.Kind.IGNORE, result.getKind());
	}

	// rust: end_turn_hides_end_turn_button
	@Test
	void endTurnHidesEndTurnButton() {
		when(game.getTurnMode()).thenReturn(TurnMode.KICKOFF_RETURN);
		module.endTurn();
		verify(communication).sendEndTurn(TurnMode.KICKOFF_RETURN);
		verify(clientData).setEndTurnButtonHidden(true);
	}

	// rust: perform_available_action_move_sends_command
	@Test
	void performAvailableActionMoveSendsCommand() {
		module.performAvailableAction(player, ClientAction.MOVE);
		verify(communication).sendActingPlayer(player, PlayerAction.MOVE, false);
	}
}
