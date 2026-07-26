package com.fumbbl.ffb.server.model;

import com.fumbbl.ffb.ApothecaryMode;
import com.fumbbl.ffb.server.step.StepParameterKey;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/model/drop_player_context_builder.rs tests.
 * DropPlayerContextBuilder fluent builder + from(existing) copy. The Rust "original" fixtures set
 * fields directly; Java has no field setters, so the originals are built via the 7-arg ctor.
 * Rust VictimStateKey -> Java StepParameterKey.
 */
public class DropPlayerContextBuilderTest {

	// rust: builder_defaults
	@Test
	public void builderDefaults() {
		DropPlayerContext ctx = DropPlayerContextBuilder.builder().build();
		assertFalse(ctx.isEndTurn());
		assertFalse(ctx.isEligibleForSafePairOfHands());
		assertFalse(ctx.isRequiresArmourBreak());
		assertFalse(ctx.isAlreadyDropped());
		assertNull(ctx.getPlayerId());
		assertNull(ctx.getApothecaryMode());
	}

	// rust: builder_set_end_turn
	@Test
	public void builderSetEndTurn() {
		DropPlayerContext ctx = DropPlayerContextBuilder.builder().endTurn(true).build();
		assertTrue(ctx.isEndTurn());
	}

	// rust: builder_set_player_id
	@Test
	public void builderSetPlayerId() {
		DropPlayerContext ctx = DropPlayerContextBuilder.builder().playerId("p-1").build();
		assertEquals("p-1", ctx.getPlayerId());
	}

	// rust: builder_set_apothecary_mode
	@Test
	public void builderSetApothecaryMode() {
		DropPlayerContext ctx = DropPlayerContextBuilder.builder().apothecaryMode(ApothecaryMode.DEFENDER).build();
		assertEquals(ApothecaryMode.DEFENDER, ctx.getApothecaryMode());
	}

	// rust: builder_set_victim_state_key
	@Test
	public void builderSetVictimStateKey() {
		DropPlayerContext ctx = DropPlayerContextBuilder.builder()
			.victimStateKey(StepParameterKey.OLD_DEFENDER_STATE).build();
		assertEquals(StepParameterKey.OLD_DEFENDER_STATE, ctx.getVictimStateKey());
	}

	// rust: builder_from_existing
	@Test
	public void builderFromExisting() {
		DropPlayerContext original = new DropPlayerContext(null, true, false, null, "orig-p", null, false);
		DropPlayerContext copied = DropPlayerContextBuilder.from(original).build();
		assertTrue(copied.isEndTurn());
		assertEquals("orig-p", copied.getPlayerId());
	}

	// rust: builder_from_then_modify
	@Test
	public void builderFromThenModify() {
		DropPlayerContext original = new DropPlayerContext(null, true, false, null, null, null, false);
		DropPlayerContext copied = DropPlayerContextBuilder.from(original)
			.endTurn(false)
			.requiresArmourBreak(true)
			.build();
		assertFalse(copied.isEndTurn());
		assertTrue(copied.isRequiresArmourBreak());
	}

	// rust: builder_additional_victim_state_keys
	@Test
	public void builderAdditionalVictimStateKeys() {
		DropPlayerContext ctx = DropPlayerContextBuilder.builder()
			.additionalVictimStateKeys(StepParameterKey.OLD_DEFENDER_STATE, StepParameterKey.OLD_PLAYER_STATE)
			.build();
		assertEquals(2, ctx.getAdditionalVictimStateKeys().length);
	}
}
