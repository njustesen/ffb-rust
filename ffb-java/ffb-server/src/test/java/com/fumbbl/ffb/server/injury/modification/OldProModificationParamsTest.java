package com.fumbbl.ffb.server.injury.modification;

import com.fumbbl.ffb.ApothecaryMode;
import com.fumbbl.ffb.injury.Block;
import com.fumbbl.ffb.injury.context.ModifiedInjuryContext;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/injury/modification/old_pro_modification_params.rs tests.
 * OldProModificationParams extends ModificationParams with selfInflicted / spottedFoul / oldValue /
 * replaceIndex value fields.
 */
public class OldProModificationParamsTest {

	private static GameState gs;

	@BeforeAll
	static void setUp() {
		gs = GameFixture.createGameState();
	}

	private OldProModificationParams params() {
		ModifiedInjuryContext ctx = new ModifiedInjuryContext();
		ctx.setApothecaryMode(ApothecaryMode.DEFENDER);
		return new OldProModificationParams(gs, ctx, new Block());
	}

	// rust: old_pro_params_defaults
	@Test
	public void oldProParamsDefaults() {
		OldProModificationParams p = params();
		assertFalse(p.isSelfInflicted());
		assertFalse(p.isSpottedFoul());
		assertEquals(0, p.getOldValue());
		assertEquals(0, p.getReplaceIndex());
	}

	// rust: old_pro_spotted_foul_detection
	@Test
	public void oldProSpottedFoulDetection() {
		OldProModificationParams p = params();
		p.setSpottedFoul(true);
		assertTrue(p.isSpottedFoul());
	}

	// rust: self_inflicted_can_be_set
	@Test
	public void selfInflictedCanBeSet() {
		OldProModificationParams p = params();
		assertFalse(p.isSelfInflicted());
		p.setSelfInflicted(true);
		assertTrue(p.isSelfInflicted());
	}

	// rust: replace_index_can_be_set
	@Test
	public void replaceIndexCanBeSet() {
		OldProModificationParams p = params();
		assertEquals(0, p.getReplaceIndex());
		p.setReplaceIndex(1);
		assertEquals(1, p.getReplaceIndex());
	}

	// rust: new_context_starts_with_no_injury
	@Test
	public void newContextStartsWithNoInjury() {
		assertNull(params().getNewContext().getInjury());
	}
}
