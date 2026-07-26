package com.fumbbl.ffb.server.injury.modification.bb2025;

import com.fumbbl.ffb.ApothecaryMode;
import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.SkillUse;
import com.fumbbl.ffb.injury.Block;
import com.fumbbl.ffb.injury.Foul;
import com.fumbbl.ffb.injury.FoulForSpp;
import com.fumbbl.ffb.injury.FoulWithChainsaw;
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
 * Mirror of ffb-rust crates/ffb-engine/src/injury/modification/bb2025/lone_fouler_modification.rs.
 * Extends bb2025 RerollArmourModification (RE_ROLL_ARMOUR) for foul injury types; extra gate = no
 * offensive/defensive foul assists. Java sources the attacker from game.getActingPlayer().getPlayer()
 * and the defender from newContext.fDefenderId. The no-assist/not-broken and broken cases port with a
 * placed acting attacker + defender (no adjacent assist-givers). Test in bb2025 package for the
 * protected tryArmourRollModification().
 * <p>
 * EXEMPT: the Rust try_armour_false_when_no_attacker_id / no_defender_id tests — Java has no
 * null-guard (attacker from the acting player, defender via lookup) and instead relies on the
 * foul-assist mechanic, so the null-id scenarios have no faithful Java path (Rust-defensive guards).
 */
public class LoneFoulerModificationTest {

	private static GameState gs;

	@BeforeAll
	static void setUp() {
		gs = GameFixture.createGameState();
		GameFixture.placePlayer(gs, "home1", 10, 5);
		GameFixture.placePlayer(gs, "away1", 10, 6);
		GameFixture.setActingPlayer(gs, "home1", PlayerAction.FOUL);
	}

	private LoneFoulerModification make() {
		return new LoneFoulerModification();
	}

	private ModificationParams params(boolean armorBroken) {
		ModifiedInjuryContext ctx = new ModifiedInjuryContext();
		ctx.setApothecaryMode(ApothecaryMode.DEFENDER);
		ctx.setDefenderId("away1");
		ctx.setArmorBroken(armorBroken);
		return new ModificationParams(gs, ctx, new Foul());
	}

	// rust: valid_foul_types
	@Test
	public void validFoulTypes() {
		LoneFoulerModification m = make();
		assertTrue(m.isValidType(new Foul()));
		assertTrue(m.isValidType(new FoulForSpp()));
		assertTrue(m.isValidType(new FoulWithChainsaw()));
		assertFalse(m.isValidType(new Block()));
	}

	// rust: skill_use_is_reroll_armour
	@Test
	public void skillUseIsReRollArmour() {
		assertEquals(SkillUse.RE_ROLL_ARMOUR, make().skillUse());
	}

	// rust: try_armour_true_when_no_assists_and_not_broken
	@Test
	public void tryArmourTrueWhenNoAssistsAndNotBroken() {
		assertTrue(make().tryArmourRollModification(params(false)));
	}

	// rust: try_armour_false_when_armor_broken_even_with_no_assists
	@Test
	public void tryArmourFalseWhenArmorBrokenEvenWithNoAssists() {
		assertFalse(make().tryArmourRollModification(params(true)));
	}
}
