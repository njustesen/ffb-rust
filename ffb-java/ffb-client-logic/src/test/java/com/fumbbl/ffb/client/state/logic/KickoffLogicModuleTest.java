package com.fumbbl.ffb.client.state.logic;

import com.fumbbl.ffb.ClientStateId;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.client.net.ClientCommunication;
import com.fumbbl.ffb.client.state.logic.interaction.InteractionResult;
import com.fumbbl.ffb.model.ActingPlayer;
import com.fumbbl.ffb.model.FieldModel;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Answers;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.mockito.junit.jupiter.MockitoSettings;
import org.mockito.quality.Strictness;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

// Mirrors ffb-rust ffb-client crates/ffb-client/src/client/state/logic/kickoff_logic_module.rs
// (Rust: mod tests), one test fn per Rust #[test] that is faithfully reproducible.
//
// NOTE: Rust's tests set the private `kicked` field directly (`module.kicked = true`). Java's
// `fKicked` has no setter/getter, so `fieldInteractionIgnoresOnceKicked` instead drives it
// indirectly by first calling `endTurn()` with an in-bounds ball coordinate (which is exactly
// how `fKicked` becomes `true` in real usage), then asserting the subsequent `fieldInteraction`
// is ignored.
@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class KickoffLogicModuleTest {

	@Mock(answer = Answers.RETURNS_DEEP_STUBS)
	FantasyFootballClient client;

	@Test
	void getIdReturnsKickoff() {
		KickoffLogicModule module = new KickoffLogicModule(client);

		assertEquals(ClientStateId.KICKOFF, module.getId());
	}

	@Test
	void fieldPeekPerformsWhenInAwayHalfAndNotKicked() {
		KickoffLogicModule module = new KickoffLogicModule(client);

		InteractionResult result = module.fieldPeek(new FieldCoordinate(20, 5));

		assertEquals(InteractionResult.Kind.PERFORM, result.getKind());
	}

	@Test
	void fieldPeekResetsOutsideAwayHalf() {
		KickoffLogicModule module = new KickoffLogicModule(client);

		InteractionResult result = module.fieldPeek(new FieldCoordinate(1, 5));

		assertEquals(InteractionResult.Kind.RESET, result.getKind());
	}

	@Test
	void fieldInteractionPlacesBallInBounds() {
		KickoffLogicModule module = new KickoffLogicModule(client);
		FieldCoordinate coordinate = new FieldCoordinate(20, 5);
		FieldModel fieldModel = client.getGame().getFieldModel();

		InteractionResult result = module.fieldInteraction(coordinate);

		assertEquals(InteractionResult.Kind.HANDLED, result.getKind());
		verify(fieldModel).setBallMoving(true);
		verify(fieldModel).setBallCoordinate(coordinate);
	}

	@Test
	void fieldInteractionIgnoresOnceKicked() {
		KickoffLogicModule module = new KickoffLogicModule(client);
		FieldCoordinate ballCoordinate = new FieldCoordinate(20, 5);
		when(client.getGame().getFieldModel().getBallCoordinate()).thenReturn(ballCoordinate);
		module.endTurn();

		InteractionResult result = module.fieldInteraction(new FieldCoordinate(20, 5));

		assertEquals(InteractionResult.Kind.IGNORE, result.getKind());
	}

	@Test
	void endTurnMarksKickedWhenBallInAwayHalf() {
		KickoffLogicModule module = new KickoffLogicModule(client);
		FieldCoordinate ballCoordinate = new FieldCoordinate(20, 5);
		when(client.getGame().getFieldModel().getBallCoordinate()).thenReturn(ballCoordinate);
		ClientCommunication communication = client.getCommunication();

		module.endTurn();

		verify(communication).sendKickoff(ballCoordinate);
		verify(client.getClientData()).setEndTurnButtonHidden(true);
	}

	@Test
	void endTurnDoesNothingWhenBallNotInAwayHalf() {
		KickoffLogicModule module = new KickoffLogicModule(client);
		FieldCoordinate ballCoordinate = new FieldCoordinate(1, 5);
		when(client.getGame().getFieldModel().getBallCoordinate()).thenReturn(ballCoordinate);
		ClientCommunication communication = client.getCommunication();

		module.endTurn();

		// Kicked remains false: a subsequent fieldInteraction still places the ball rather
		// than being ignored, indirectly proving fKicked was not set.
		InteractionResult result = module.fieldInteraction(new FieldCoordinate(20, 5));
		assertEquals(InteractionResult.Kind.HANDLED, result.getKind());
	}

	@Test
	void turnIsEndingReflectsBallPosition() {
		KickoffLogicModule module = new KickoffLogicModule(client);
		FieldCoordinate ballCoordinate = new FieldCoordinate(20, 5);
		when(client.getGame().getFieldModel().getBallCoordinate()).thenReturn(ballCoordinate);

		assertTrue(module.turnIsEnding());
	}

	@Test
	void actionContextPanics() {
		KickoffLogicModule module = new KickoffLogicModule(client);
		ActingPlayer actingPlayer = new ActingPlayer(null);

		assertThrows(UnsupportedOperationException.class, () -> module.actionContext(actingPlayer));
	}
}
