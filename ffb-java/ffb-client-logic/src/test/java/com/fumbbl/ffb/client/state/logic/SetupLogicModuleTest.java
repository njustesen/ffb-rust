package com.fumbbl.ffb.client.state.logic;

import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.ClientStateId;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.client.state.logic.interaction.InteractionResult;
import com.fumbbl.ffb.model.ActingPlayer;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.net.NetCommand;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.commands.ServerCommandTeamSetupList;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Answers;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.mockito.junit.jupiter.MockitoSettings;
import org.mockito.quality.Strictness;

import static org.junit.jupiter.api.Assertions.assertDoesNotThrow;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class SetupLogicModuleTest {

	@Mock(answer = Answers.RETURNS_DEEP_STUBS)
	private FantasyFootballClient client;

	@Mock
	private ActingPlayer actingPlayer;

	@Mock
	private Player<?> player;

	@Mock
	private NetCommand nonMatchingCommand;

	@Test
	public void testGetIdReturnsSetup() {
		SetupLogicModule module = new SetupLogicModule(client);
		assertEquals(ClientStateId.SETUP, module.getId());
	}

	@Test
	public void testAvailableActionsIsEmpty() {
		assertTrue(new SetupLogicModule(client).availableActions().isEmpty());
	}

	@Test
	public void testPerformAvailableActionIsNoOp() {
		SetupLogicModule module = new SetupLogicModule(client);
		assertDoesNotThrow(() -> module.performAvailableAction(player, ClientAction.MOVE));
	}

	@Test
	public void testUseTurnModeIsFalse() {
		assertFalse(new SetupLogicModule(client).useTurnMode());
	}

	@Test
	public void testActionContextThrows() {
		SetupLogicModule module = new SetupLogicModule(client);
		UnsupportedOperationException exception = assertThrows(UnsupportedOperationException.class,
			() -> module.actionContext(actingPlayer));
		assertEquals("actionContext for acting player is not supported in setup context", exception.getMessage());
	}

	@Test
	public void testSquareIsValidForSetupChecksHalfHomeOrReserveColumn() {
		SetupLogicModule module = new SetupLogicModule(client);
		assertTrue(module.squareIsValidForSetup(new FieldCoordinate(5, 5)));
		assertFalse(module.squareIsValidForSetup(new FieldCoordinate(20, 5)));
		assertTrue(module.squareIsValidForSetup(new FieldCoordinate(FieldCoordinate.RSV_HOME_X, 3)));
	}

	@Test
	public void testSquareIsEmptyTrueWhenNoPlayerOnSquare() {
		given(client.getGame().getFieldModel().getPlayer(new FieldCoordinate(1, 1))).willReturn(null);
		SetupLogicModule module = new SetupLogicModule(client);
		assertTrue(module.squareIsEmpty(new FieldCoordinate(1, 1)));
	}

	@Test
	public void testSquareIsEmptyFalseWhenPlayerOnSquare() {
		given(client.getGame().getFieldModel().getPlayer(new FieldCoordinate(2, 2))).willReturn((com.fumbbl.ffb.model.Player) player);
		SetupLogicModule module = new SetupLogicModule(client);
		assertFalse(module.squareIsEmpty(new FieldCoordinate(2, 2)));
	}

	@Test
	public void testHandleCommandHandledForServerTeamSetupList() {
		ServerCommandTeamSetupList command = new ServerCommandTeamSetupList(new String[]{"Wide"});
		SetupLogicModule module = new SetupLogicModule(client);
		InteractionResult result = module.handleCommand(command, true);
		assertEquals(InteractionResult.Kind.HANDLED, result.getKind());
	}

	@Test
	public void testHandleCommandIgnoresNonMatchingCommand() {
		given(nonMatchingCommand.getId()).willReturn(NetCommandId.CLIENT_USE_SKILL);
		SetupLogicModule module = new SetupLogicModule(client);
		InteractionResult result = module.handleCommand(nonMatchingCommand, true);
		assertEquals(InteractionResult.Kind.IGNORE, result.getKind());
	}
}
