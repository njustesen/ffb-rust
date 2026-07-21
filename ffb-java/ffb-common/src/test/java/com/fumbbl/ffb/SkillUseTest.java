package com.fumbbl.ffb;

import com.fumbbl.ffb.factory.SkillUseFactory;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/model/skill_use.rs for {@link SkillUse}.
 */
public class SkillUseTest {

	private final SkillUseFactory factory = new SkillUseFactory();

	@Test
	public void forNameCaseInsensitive() {
		assertEquals(SkillUse.WOULD_NOT_HELP, factory.forName("wouldNotHelp"));
		assertEquals(SkillUse.WOULD_NOT_HELP, factory.forName("WOULDNOTHELP"));
		assertNull(factory.forName("invalid"));
	}

	@Test
	public void forNameReturnsNoneForUnknown() {
		assertNull(factory.forName("nonexistentSkillUse"));
	}

	@Test
	public void allVariantsHaveNonEmptyDescription() {
		SkillUse[] variants = {
			SkillUse.STOP_OPPONENT, SkillUse.CATCH_BALL, SkillUse.STEAL_BALL,
			SkillUse.RE_ROLL_ARMOUR, SkillUse.ADD_BLOCK_DIE, SkillUse.BULLSEYE,
			SkillUse.GRANT_SKILL_TO_TEAM_MATE, SkillUse.AVOID_DODGING,
			SkillUse.GAIN_CLAWS_FOR_BLITZ, SkillUse.SAVED_FUMBLE_BOMB
		};
		for (SkillUse variant : variants) {
			assertFalse(variant.getDescription(null).isEmpty(), variant + " has an empty description");
		}
	}

}
