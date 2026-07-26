package com.fumbbl.ffb.server.injury.modification.bb2025;

import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.SkillUse;
import com.fumbbl.ffb.injury.Block;
import com.fumbbl.ffb.injury.Stab;
import com.fumbbl.ffb.injury.StabForSpp;
import com.fumbbl.ffb.injury.context.InjuryContext;
import com.fumbbl.ffb.model.Game;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/injury/modification/bb2025/toxin_connoisseur_modification.rs.
 * bb2025 ToxinConnoisseurModification (ADD_INJURY_MODIFIER, valid Stab + StabForSpp). tryInjuryModification
 * is !isCasualty() with no acting-player deref, so the game arg is null and every test ports; skillUse()
 * is declared in this bb2025 class (reachable from this same-package test).
 */
public class ToxinConnoisseurModificationTest {

	private ToxinConnoisseurModification make() {
		return new ToxinConnoisseurModification();
	}

	// rust: valid_types
	@Test
	public void validTypes() {
		ToxinConnoisseurModification m = make();
		assertTrue(m.isValidType(new Stab()));
		assertTrue(m.isValidType(new StabForSpp()));
		assertFalse(m.isValidType(new Block()));
	}

	// rust: try_injury_false_when_casualty
	@Test
	public void tryInjuryFalseWhenCasualty() {
		InjuryContext ctx = new InjuryContext();
		ctx.setInjury(new PlayerState(PlayerState.SERIOUS_INJURY));
		assertFalse(make().tryInjuryModification((Game) null, ctx, new Stab()));
	}

	// rust: try_injury_true_when_not_casualty
	@Test
	public void tryInjuryTrueWhenNotCasualty() {
		assertTrue(make().tryInjuryModification((Game) null, new InjuryContext(), new Stab()));
	}

	// rust: skill_use_is_add_injury_modifier
	@Test
	public void skillUseIsAddInjuryModifier() {
		assertEquals(SkillUse.ADD_INJURY_MODIFIER, make().skillUse());
	}

	// rust: skill_id_starts_as_none
	@Test
	public void skillIdStartsAsNone() {
		assertNull(make().getSkill());
	}
}
