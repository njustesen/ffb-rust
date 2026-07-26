package com.fumbbl.ffb.server.injury.modification;

import com.fumbbl.ffb.SkillUse;
import com.fumbbl.ffb.injury.Block;
import com.fumbbl.ffb.injury.Chainsaw;
import com.fumbbl.ffb.injury.Foul;
import com.fumbbl.ffb.injury.Stab;
import com.fumbbl.ffb.injury.context.InjuryContext;
import com.fumbbl.ffb.model.Game;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/injury/modification/savage_mauling_modification.rs tests
 * (valid_type / skill_use / non-casualty subset). tryInjuryModification short-circuits to true at
 * !isCasualty() (no player deref), so the not-casualty case ports with game=null. The casualty-path
 * tests (spotted-foul, same-team, animal-savagery) need placed players + teams and are deferred;
 * is_spotted_foul is a private Java helper (exempt).
 */
public class SavageMaulingModificationTest {

	private SavageMaulingModification make() {
		return new SavageMaulingModification();
	}

	// rust: valid_types
	@Test
	public void validTypes() {
		SavageMaulingModification m = make();
		assertTrue(m.isValidType(new Block()));
		assertTrue(m.isValidType(new Foul()));
		assertTrue(m.isValidType(new Stab()));
		assertFalse(m.isValidType(new Chainsaw()));
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

	// rust: try_injury_true_when_not_casualty
	@Test
	public void tryInjuryTrueWhenNotCasualty() {
		assertTrue(make().tryInjuryModification((Game) null, new InjuryContext(), new Block()));
	}
}
