package com.fumbbl.ffb.server.model;

import com.fumbbl.ffb.ApothecaryMode;
import com.fumbbl.ffb.server.InjuryResult;
import com.fumbbl.ffb.server.step.StepParameterKey;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/drop_player_context.rs (model/drop_player_context.rs) tests.
 * The Rust with_injury(ir, playerId, apothecaryMode, eligible) shorthand maps to the 7-arg Java ctor
 * (injuryResult, endTurn, eligibleForSafePairOfHands, label, playerId, apothecaryMode, requiresArmourBreak).
 * The Rust VictimStateKey enum maps to Java StepParameterKey (only the PlayerState-carrying keys).
 */
public class DropPlayerContextTest {

	// rust: new_has_default_fields
	@Test
	public void newHasDefaultFields() {
		DropPlayerContext ctx = new DropPlayerContext();
		assertFalse(ctx.isEndTurn());
		assertFalse(ctx.isEligibleForSafePairOfHands());
		assertFalse(ctx.isRequiresArmourBreak());
		assertFalse(ctx.isAlreadyDropped());
		assertFalse(ctx.isModifiedInjuryEndsTurn());
		assertFalse(ctx.isEndTurnWithoutKnockdown());
		assertNull(ctx.getLabel());
		assertNull(ctx.getPlayerId());
		assertNull(ctx.getApothecaryMode());
		assertNull(ctx.getVictimStateKey());
		assertNull(ctx.getAdditionalVictimStateKeys());
		assertNull(ctx.getInjuryResult());
	}

	// rust: with_injury_sets_fields
	@Test
	public void withInjurySetsFields() {
		InjuryResult injuryResult = new InjuryResult();
		DropPlayerContext ctx = new DropPlayerContext(injuryResult, false, true, null, "player-1",
			ApothecaryMode.ATTACKER, false);
		assertEquals("player-1", ctx.getPlayerId());
		assertEquals(ApothecaryMode.ATTACKER, ctx.getApothecaryMode());
		assertTrue(ctx.isEligibleForSafePairOfHands());
		assertEquals(injuryResult, ctx.getInjuryResult());
	}

	// rust: victim_state_key_variants_exist
	@Test
	public void victimStateKeyVariantsExist() {
		assertNotNull(StepParameterKey.OLD_DEFENDER_STATE);
		assertNotNull(StepParameterKey.OLD_PLAYER_STATE);
		assertNotNull(StepParameterKey.THROWN_PLAYER_STATE);
		assertNotNull(StepParameterKey.KICKED_PLAYER_STATE);
	}

	// rust: with_injury_not_eligible_for_safe_pair_of_hands
	@Test
	public void withInjuryNotEligibleForSafePairOfHands() {
		InjuryResult injuryResult = new InjuryResult();
		DropPlayerContext ctx = new DropPlayerContext(injuryResult, false, false, null, "p2",
			ApothecaryMode.DEFENDER, false);
		assertFalse(ctx.isEligibleForSafePairOfHands());
		assertEquals(ApothecaryMode.DEFENDER, ctx.getApothecaryMode());
	}

	// rust: default_same_as_new
	@Test
	public void defaultSameAsNew() {
		DropPlayerContext a = new DropPlayerContext();
		DropPlayerContext b = new DropPlayerContext();
		assertEquals(a.isEndTurn(), b.isEndTurn());
		assertEquals(a.getPlayerId(), b.getPlayerId());
		assertEquals(a.getAdditionalVictimStateKeys(), b.getAdditionalVictimStateKeys());
	}

	// rust: victim_state_key_variants_are_distinct
	@Test
	public void victimStateKeyVariantsAreDistinct() {
		assertNotEquals(StepParameterKey.OLD_DEFENDER_STATE, StepParameterKey.OLD_PLAYER_STATE);
		assertNotEquals(StepParameterKey.THROWN_PLAYER_STATE, StepParameterKey.KICKED_PLAYER_STATE);
		assertNotEquals(StepParameterKey.OLD_DEFENDER_STATE, StepParameterKey.THROWN_PLAYER_STATE);
	}

	// rust: victim_state_key_copy_semantics
	@Test
	public void victimStateKeyCopySemantics() {
		StepParameterKey a = StepParameterKey.THROWN_PLAYER_STATE;
		StepParameterKey b = a;
		assertEquals(a, b);
	}

	// rust: with_injury_leaves_other_booleans_false
	@Test
	public void withInjuryLeavesOtherBooleansFalse() {
		InjuryResult injuryResult = new InjuryResult();
		DropPlayerContext ctx = new DropPlayerContext(injuryResult, false, true, null, "p3",
			ApothecaryMode.ATTACKER, false);
		assertFalse(ctx.isEndTurn());
		assertFalse(ctx.isRequiresArmourBreak());
		assertFalse(ctx.isAlreadyDropped());
		assertFalse(ctx.isModifiedInjuryEndsTurn());
		assertFalse(ctx.isEndTurnWithoutKnockdown());
		assertNull(ctx.getLabel());
		assertNull(ctx.getVictimStateKey());
		assertNull(ctx.getAdditionalVictimStateKeys());
	}
}
