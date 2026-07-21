package com.fumbbl.ffb.client.state.logic.interaction;

import com.fumbbl.ffb.ClientStateId;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.client.state.logic.ClientAction;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertSame;

class InteractionResultTest {

	@Test
	public void testIgnoreHasIgnoreKind() {
		assertEquals(InteractionResult.Kind.IGNORE, InteractionResult.ignore().getKind());
	}

	@Test
	public void testHandledHasHandledKind() {
		assertEquals(InteractionResult.Kind.HANDLED, InteractionResult.handled().getKind());
	}

	@Test
	public void testPerformHasPerformKind() {
		assertEquals(InteractionResult.Kind.PERFORM, InteractionResult.perform().getKind());
	}

	@Test
	public void testInvalidHasInvalidKind() {
		assertEquals(InteractionResult.Kind.INVALID, InteractionResult.invalid().getKind());
	}

	@Test
	public void testResetHasResetKind() {
		assertEquals(InteractionResult.Kind.RESET, InteractionResult.reset().getKind());
	}

	@Test
	public void testPreviewThrowHasPreviewThrowKind() {
		assertEquals(InteractionResult.Kind.PREVIEW_THROW, InteractionResult.previewThrow().getKind());
	}

	@Test
	public void testDelegateHasDelegateKindAndPayload() {
		InteractionResult result = InteractionResult.delegate(ClientStateId.MOVE);
		assertEquals(InteractionResult.Kind.DELEGATE, result.getKind());
		assertSame(ClientStateId.MOVE, result.getDelegate());
	}

	@Test
	public void testSelectActionHasSelectActionKindAndPayload() {
		ActionContext ctx = new ActionContext();
		ctx.add(ClientAction.MOVE);
		InteractionResult result = InteractionResult.selectAction(ctx);
		assertEquals(InteractionResult.Kind.SELECT_ACTION, result.getKind());
		assertEquals(1, result.getActionContext().getActions().size());
	}

	@Test
	public void testWithCoordinateSetsPayload() {
		InteractionResult result = InteractionResult.perform().with(new FieldCoordinate(1, 2));
		assertEquals(new FieldCoordinate(1, 2), result.getCoordinate());
	}
}
