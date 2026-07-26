package com.fumbbl.ffb.server.injury.modification;

import com.fumbbl.ffb.ApothecaryMode;
import com.fumbbl.ffb.SkillUse;
import com.fumbbl.ffb.injury.Block;
import com.fumbbl.ffb.injury.Stab;
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
 * Mirror of ffb-rust crates/ffb-engine/src/injury/modification/crushing_blow_modification.rs tests.
 * Armour-side (ADD_ARMOUR_MODIFIER, valid Block). tryArmourRollModification short-circuits to false
 * when armour is broken without an affects-either MB modifier (before the acting-player deref), so
 * that case ports with a broken-armour ModificationParams. Same-package test for protected methods.
 * <p>
 * EXEMPT: the Rust try_armour_false_when_not_broken_but_no_active_player test — the not-broken branch
 * dereferences game.getActingPlayer().getPlayer() (no null-guard; always present in production).
 */
public class CrushingBlowModificationTest {

	private static GameState gs;

	@BeforeAll
	static void setUp() {
		gs = GameFixture.createGameState();
	}

	private CrushingBlowModification make() {
		return new CrushingBlowModification();
	}

	// rust: valid_type_is_block
	@Test
	public void validTypeIsBlock() {
		assertTrue(make().isValidType(new Block()));
		assertFalse(make().isValidType(new Stab()));
	}

	// rust: try_armour_false_when_broken_without_mb
	@Test
	public void tryArmourFalseWhenBrokenWithoutMb() {
		ModifiedInjuryContext ctx = new ModifiedInjuryContext();
		ctx.setApothecaryMode(ApothecaryMode.DEFENDER);
		ctx.setArmorBroken(true);
		ModificationParams params = new ModificationParams(gs, ctx, new Block());
		assertFalse(make().tryArmourRollModification(params));
	}

	// rust: skill_use_is_add_armour_modifier
	@Test
	public void skillUseIsAddArmourModifier() {
		assertEquals(SkillUse.ADD_ARMOUR_MODIFIER, make().skillUse());
	}

	// rust: skill_id_starts_as_none
	@Test
	public void skillIdStartsAsNone() {
		assertNull(make().getSkill());
	}
}
