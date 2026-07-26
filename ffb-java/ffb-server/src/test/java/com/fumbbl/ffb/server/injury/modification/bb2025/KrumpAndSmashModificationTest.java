package com.fumbbl.ffb.server.injury.modification.bb2025;

import com.fumbbl.ffb.ApothecaryMode;
import com.fumbbl.ffb.SkillUse;
import com.fumbbl.ffb.injury.Block;
import com.fumbbl.ffb.injury.Stab;
import com.fumbbl.ffb.injury.context.ModifiedInjuryContext;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.injury.modification.ModificationParams;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/injury/modification/bb2025/krump_and_smash_modification.rs.
 * Extends bb2025 RerollArmourModification (RE_ROLL_ARMOUR, valid Block); tryArmourRollModification is
 * !isArmorBroken() with no acting-player deref, so all branches port. Test lives in the bb2025 package
 * to reach the protected skillUse()/tryArmourRollModification()/modifyArmourInternal().
 */
public class KrumpAndSmashModificationTest {

	private static GameState gs;

	@BeforeAll
	static void setUp() {
		gs = GameFixture.createGameState();
	}

	private KrumpAndSmashModification make() {
		return new KrumpAndSmashModification();
	}

	private ModificationParams params(boolean armorBroken) {
		ModifiedInjuryContext ctx = new ModifiedInjuryContext();
		ctx.setApothecaryMode(ApothecaryMode.DEFENDER);
		ctx.setArmorBroken(armorBroken);
		return new ModificationParams(gs, ctx, new Block());
	}

	// rust: valid_type_is_block
	@Test
	public void validTypeIsBlock() {
		assertTrue(make().isValidType(new Block()));
		assertFalse(make().isValidType(new Stab()));
	}

	// rust: skill_use_is_reroll_armour
	@Test
	public void skillUseIsReRollArmour() {
		assertEquals(SkillUse.RE_ROLL_ARMOUR, make().skillUse());
	}

	// rust: try_armour_false_when_broken
	@Test
	public void tryArmourFalseWhenBroken() {
		assertFalse(make().tryArmourRollModification(params(true)));
	}

	// rust: try_armour_true_when_not_broken
	@Test
	public void tryArmourTrueWhenNotBroken() {
		assertTrue(make().tryArmourRollModification(params(false)));
	}

	// rust: modify_armour_internal_returns_true
	@Test
	public void modifyArmourInternalReturnsTrue() {
		ModificationParams params = params(false);
		assertTrue(make().modifyArmourInternal(params));
		int[] armorRoll = params.getNewContext().getArmorRoll();
		assertEquals(2, armorRoll.length);
		assertTrue(armorRoll[0] >= 1 && armorRoll[0] <= 6);
		assertTrue(armorRoll[1] >= 1 && armorRoll[1] <= 6);
	}
}
