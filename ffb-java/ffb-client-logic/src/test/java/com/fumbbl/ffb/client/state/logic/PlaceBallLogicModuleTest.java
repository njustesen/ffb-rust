package com.fumbbl.ffb.client.state.logic;

import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.MoveSquare;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.client.state.logic.interaction.InteractionResult;
import com.fumbbl.ffb.model.ActingPlayer;
import com.fumbbl.ffb.model.FieldModel;
import com.fumbbl.ffb.model.Game;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Answers;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.mockito.junit.jupiter.MockitoSettings;
import org.mockito.quality.Strictness;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

/**
 * Ported from {@code ffb-rust/crates/ffb-client/src/client/state/logic/place_ball_logic_module.rs}'s
 * {@code #[cfg(test)] mod tests}, against the real {@link PlaceBallLogicModule}.
 */
@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class PlaceBallLogicModuleTest {

	@Mock(answer = Answers.RETURNS_DEEP_STUBS)
	private FantasyFootballClient client;

	@Mock
	private Game game;

	@Mock
	private FieldModel fieldModel;

	@Mock
	private ActingPlayer actingPlayer;

	private PlaceBallLogicModule module;

	@BeforeEach
	void setUp() {
		when(client.getGame()).thenReturn(game);
		when(game.getFieldModel()).thenReturn(fieldModel);

		module = new PlaceBallLogicModule(client);
	}

	// Rust: field_interaction_ignores_non_move_square
	@Test
	void fieldInteractionIgnoresNonMoveSquare() {
		FieldCoordinate coordinate = new FieldCoordinate(1, 1);
		when(fieldModel.getMoveSquare(coordinate)).thenReturn(null);

		InteractionResult result = module.fieldInteraction(coordinate);

		assertEquals(InteractionResult.Kind.IGNORE, result.getKind());
	}

	// Rust: field_interaction_handles_move_square_and_sends_coordinate
	@Test
	void fieldInteractionHandlesMoveSquareAndSendsCoordinate() {
		FieldCoordinate coordinate = new FieldCoordinate(3, 4);
		MoveSquare moveSquare = new MoveSquare(coordinate, 1, 0);
		when(fieldModel.getMoveSquare(coordinate)).thenReturn(moveSquare);

		InteractionResult result = module.fieldInteraction(coordinate);

		assertEquals(InteractionResult.Kind.HANDLED, result.getKind());
		verify(client.getCommunication()).sendFieldCoordinate(coordinate);
	}

	// Rust: field_peek_returns_perform_for_move_square_and_reset_otherwise
	@Test
	void fieldPeekReturnsPerformForMoveSquareAndResetOtherwise() {
		FieldCoordinate coordinate = new FieldCoordinate(5, 5);
		when(fieldModel.getMoveSquare(coordinate)).thenReturn(null);
		assertEquals(InteractionResult.Kind.RESET, module.fieldPeek(coordinate).getKind());

		MoveSquare moveSquare = new MoveSquare(coordinate, 1, 0);
		when(fieldModel.getMoveSquare(coordinate)).thenReturn(moveSquare);
		assertEquals(InteractionResult.Kind.PERFORM, module.fieldPeek(coordinate).getKind());
	}

	// Rust: action_context_panics
	@Test
	void actionContextThrowsUnsupportedOperationException() {
		assertThrows(UnsupportedOperationException.class, () -> module.actionContext(actingPlayer));
	}
}
