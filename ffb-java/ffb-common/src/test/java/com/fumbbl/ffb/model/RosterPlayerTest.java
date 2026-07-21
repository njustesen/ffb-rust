package com.fumbbl.ffb.model;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.PlayerGender;
import com.fumbbl.ffb.PlayerType;
import com.fumbbl.ffb.factory.SkillFactory;
import com.fumbbl.ffb.model.skill.Skill;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/model/roster_player.rs for {@link RosterPlayer}.
 */
public class RosterPlayerTest {

	private Skill skill(String name) {
		SkillFactory factory = NetCommandTestUtil.gameSource().getFactory(FactoryType.Factory.SKILL);
		return factory.forName(name);
	}

	private RosterPlayer makeRosterPlayer() {
		RosterPlayer p = new RosterPlayer();
		p.setId("test_p");
		p.setName("Test Player");
		p.setNr(5);
		p.setPositionId("lineman");
		p.setType(PlayerType.REGULAR);
		p.setGender(PlayerGender.MALE);
		p.setMovement(6);
		p.setStrength(3);
		p.setAgility(3);
		p.setPassing(4);
		p.setArmour(8);
		p.setPlayerStatus(PlayerStatus.ACTIVE);
		return p;
	}

	@Test
	public void addAndRemoveSkill() {
		RosterPlayer p = makeRosterPlayer();
		Skill loner = skill("Loner");
		p.addSkill(loner);
		assertTrue(p.has(loner));
		p.removeSkill(loner);
		assertFalse(p.has(loner));
	}

}
