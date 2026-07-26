package com.fumbbl.ffb.server.injury.modification;

import com.fumbbl.ffb.ApothecaryMode;
import com.fumbbl.ffb.SkillUse;
import com.fumbbl.ffb.injury.Block;
import com.fumbbl.ffb.injury.Chainsaw;
import com.fumbbl.ffb.injury.context.ModifiedInjuryContext;
import com.fumbbl.ffb.server.fixture.GameFixture;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/injury/modification/ghostly_flames_modification.rs tests.
 * ARMOUR-side (INCREASE_CHAINSAW_DAMAGE, valid Chainsaw). tryArmourRollModification short-circuits on
 * isArmorBroken() before the blitzing-acting-player checks, so the armour-broken case ports with a
 * GameFixture params. Same-package test for protected skillUse()/tryArmourRollModification().
 * <p>
 * EXEMPT: the Rust try_armour_false_when_not_broken_but_no_active_player test — Java's
 * tryArmourRollModification dereferences actingPlayer.getPlayerAction().isBlitzing(); the
 * no-acting-player scenario has no faithful Java path (Rust-defensive guard).
 */
public class GhostlyFlamesModificationTest {

	private GhostlyFlamesModification make() {
		return new GhostlyFlamesModification();
	}

	private ModificationParams brokenParams() {
		ModifiedInjuryContext ctx = new ModifiedInjuryContext();
		ctx.setApothecaryMode(ApothecaryMode.ATTACKER);
		ctx.setArmorBroken(true);
		return new ModificationParams(GameFixture.createGameState(), ctx, new Chainsaw());
	}

	// rust: valid_type_is_chainsaw
	@Test
	public void validTypeIsChainsaw() {
		assertTrue(make().isValidType(new Chainsaw()));
		assertFalse(make().isValidType(new Block()));
	}

	// rust: try_armour_false_when_broken
	@Test
	public void tryArmourFalseWhenBroken() {
		assertFalse(make().tryArmourRollModification(brokenParams()));
	}

	// rust: skill_use_is_increase_chainsaw_damage
	@Test
	public void skillUseIsIncreaseChainsawDamage() {
		assertEquals(SkillUse.INCREASE_CHAINSAW_DAMAGE, make().skillUse());
	}

	// rust: skill_id_starts_as_none
	@Test
	public void skillIdStartsAsNone() {
		assertNull(make().getSkill());
	}
}
