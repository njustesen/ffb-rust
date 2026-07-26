package com.fumbbl.ffb.server.injury.modification;

import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.SkillUse;
import com.fumbbl.ffb.injury.Block;
import com.fumbbl.ffb.injury.Stab;
import com.fumbbl.ffb.injury.context.InjuryContext;
import com.fumbbl.ffb.model.Game;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/injury/modification/master_assassin_modification.rs tests.
 * Base MasterAssassinModification (RE_ROLL_INJURY). tryInjuryModification is `!isCasualty()` — no
 * acting-player deref, so the game arg is unused (passed null) and both branches port cleanly.
 * Same-package test to reach protected skillUse()/tryInjuryModification().
 */
public class MasterAssassinModificationTest {

	private MasterAssassinModification make() {
		return new MasterAssassinModification();
	}

	// rust: valid_type_is_stab
	@Test
	public void validTypeIsStab() {
		assertTrue(make().isValidType(new Stab()));
		assertFalse(make().isValidType(new Block()));
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
		InjuryContext ctx = new InjuryContext();
		assertTrue(make().tryInjuryModification((Game) null, ctx, new Stab()));
	}

	// rust: skill_use_is_re_roll_injury
	@Test
	public void skillUseIsReRollInjury() {
		assertEquals(SkillUse.RE_ROLL_INJURY, make().skillUse());
	}

	// rust: skill_id_starts_as_none
	@Test
	public void skillIdStartsAsNone() {
		assertNull(make().getSkill());
	}
}
