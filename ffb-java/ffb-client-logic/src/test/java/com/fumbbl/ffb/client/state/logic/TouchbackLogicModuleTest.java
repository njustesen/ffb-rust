package com.fumbbl.ffb.client.state.logic;

import com.fumbbl.ffb.ClientStateId;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.client.state.logic.interaction.InteractionResult;
import com.fumbbl.ffb.model.ActingPlayer;
import com.fumbbl.ffb.model.Player;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Answers;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.mockito.junit.jupiter.MockitoSettings;
import org.mockito.quality.Strictness;

import static org.junit.jupiter.api.Assertions.assertDoesNotThrow;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class TouchbackLogicModuleTest {

	@Mock(answer = Answers.RETURNS_DEEP_STUBS)
	private FantasyFootballClient client;

	@Mock
	private ActingPlayer actingPlayer;

	@Mock
	private Player<?> player;

	@Test
	public void testGetIdReturnsTouchback() {
		TouchbackLogicModule module = new TouchbackLogicModule(client);
		assertEquals(ClientStateId.TOUCHBACK, module.getId());
	}

	@Test
	public void testAvailableActionsIsEmpty() {
		assertTrue(new TouchbackLogicModule(client).availableActions().isEmpty());
	}

	@Test
	public void testPerformAvailableActionIsNoOp() {
		TouchbackLogicModule module = new TouchbackLogicModule(client);
		assertDoesNotThrow(() -> module.performAvailableAction(player, ClientAction.MOVE));
	}

	@Test
	public void testFieldPeekResetsBeforeSetUp() {
		// fTouchbackToAnyField defaults to false until setUp() runs.
		TouchbackLogicModule module = new TouchbackLogicModule(client);
		assertEquals(InteractionResult.Kind.RESET, module.fieldPeek(new FieldCoordinate(1, 1)).getKind());
	}

	@Test
	public void testSetUpKeepsTouchbackToAnyFieldWhenNoHomePlayers() {
		given(client.getGame().getTeamHome().getPlayers()).willReturn(new Player<?>[0]);

		TouchbackLogicModule module = new TouchbackLogicModule(client);
		module.setUp();

		assertEquals(InteractionResult.Kind.PERFORM, module.fieldPeek(new FieldCoordinate(1, 1)).getKind());
	}

	@Test
	public void testFieldInteractionHandledOnlyAfterTouchbackToAnyField() {
		TouchbackLogicModule module = new TouchbackLogicModule(client);
		assertEquals(InteractionResult.Kind.IGNORE,
			module.fieldInteraction(new FieldCoordinate(1, 1)).getKind());

		given(client.getGame().getTeamHome().getPlayers()).willReturn(new Player<?>[0]);
		module.setUp();

		assertEquals(InteractionResult.Kind.HANDLED,
			module.fieldInteraction(new FieldCoordinate(1, 1)).getKind());
	}

	@Test
	public void testActionContextThrows() {
		TouchbackLogicModule module = new TouchbackLogicModule(client);
		UnsupportedOperationException exception = assertThrows(UnsupportedOperationException.class,
			() -> module.actionContext(actingPlayer));
		assertEquals("actionContext for acting player is not supported in touchback context", exception.getMessage());
	}

	// SKIPPED: playerPeek(Player)/playerInteraction(Player) short-circuit on fTouchbackToAnyField
	// like fieldPeek/fieldInteraction above, but isPlayerSelectable(Player) additionally requires
	// a live FieldModel/PlayerState/Team graph once fTouchbackToAnyField is false — out of scope
	// per the conservative directive (building live game state to drive playerInteraction()).
}
