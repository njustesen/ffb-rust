package com.fumbbl.ffb.model;

import com.eclipsesource.json.JsonValue;
import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.PlayerGender;
import com.fumbbl.ffb.PlayerType;
import com.fumbbl.ffb.factory.SkillFactory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.model.skill.Skill;
import com.fumbbl.ffb.model.skill.SkillWithValue;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import java.util.Collections;
import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/model/player.rs for {@link Player}.
 */
public class PlayerModelTest {

	private Skill skill(String name) {
		SkillFactory factory = NetCommandTestUtil.gameSource().getFactory(FactoryType.Factory.SKILL);
		return factory.forName(name);
	}

	private RosterPlayer testPlayer() {
		RosterPlayer p = new RosterPlayer();
		p.setId("p1");
		p.setName("Joe");
		p.setNr(1);
		p.setPositionId("lineman");
		p.setType(PlayerType.REGULAR);
		p.setGender(PlayerGender.MALE);
		p.setMovement(6);
		p.setStrength(3);
		p.setAgility(3);
		p.setPassing(4);
		p.setArmour(8);
		return p;
	}

	@Test
	public void serdeRoundTrip() {
		RosterPlayer p = testPlayer();
		JsonValue json = p.toJsonValue();
		RosterPlayer back = new RosterPlayer().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals(p.getId(), back.getId());
		assertEquals(p.getMovement(), back.getMovement());
	}

	@Test
	public void hasSkillFalseWhenEmpty() {
		RosterPlayer p = testPlayer();
		assertFalse(p.has(skill("Block")));
	}

	@Test
	public void hasSkillTrueForStartingSkill() {
		RosterPlayer p = testPlayer();
		p.addSkill(skill("Block"));
		assertTrue(p.has(skill("Block")));
		assertFalse(p.has(skill("Tackle")));
	}

	@Test
	public void hasSkillTrueForExtraSkill() {
		RosterPlayer p = testPlayer();
		p.addSkill(skill("Dodge"));
		assertTrue(p.has(skill("Dodge")));
	}

	@Test
	public void movementWithModifiersReturnsBase() {
		assertEquals(6, testPlayer().getMovementWithModifiers());
	}

	@Test
	public void strengthWithModifiersReturnsBase() {
		assertEquals(3, testPlayer().getStrengthWithModifiers());
	}

	@Test
	public void agilityWithModifiersReturnsBase() {
		assertEquals(3, testPlayer().getAgilityWithModifiers());
	}

	@Test
	public void armourWithModifiersReturnsBase() {
		assertEquals(8, testPlayer().getArmourWithModifiers());
	}

	@Test
	public void passingWithModifiersReturnsBase() {
		assertEquals(4, testPlayer().getPassingWithModifiers());
	}

	@Test
	public void hasSkillTrueForTemporarySkill() {
		RosterPlayer p = testPlayer();
		p.addTemporarySkills("SOURCE", Collections.singleton(new SkillWithValue(skill("Sprint"), null)));
		assertTrue(p.has(skill("Sprint")));
		assertFalse(p.has(skill("Block")));
	}

	@Test
	public void allSkillIdsIteratesAllThreeSkillLists() {
		RosterPlayer p = testPlayer();
		p.addSkill(skill("Block"));
		p.addSkill(skill("Dodge"));
		p.addTemporarySkills("SOURCE", Collections.singleton(new SkillWithValue(skill("Sprint"), null)));
		List<Skill> ids = p.getSkillsIncludingTemporaryOnesWithDuplicates();
		assertEquals(3, ids.size());
		assertTrue(ids.contains(skill("Block")));
		assertTrue(ids.contains(skill("Dodge")));
		assertTrue(ids.contains(skill("Sprint")));
	}

	@Test
	public void hasSkillPropertyReturnsTrueForMatchingSkill() {
		RosterPlayer p = testPlayer();
		p.addSkill(skill("Block"));
		assertTrue(p.hasSkillProperty(NamedProperties.preventFallOnBothDown));
		assertFalse(p.hasSkillProperty(NamedProperties.canLeap));
	}

	@Test
	public void hasSkillPropertyFalseWhenNoSkills() {
		RosterPlayer p = testPlayer();
		assertFalse(p.hasSkillProperty(NamedProperties.preventFallOnBothDown));
	}

	@Test
	public void hasSkillPropertyChecksAllSkillLists() {
		RosterPlayer p = testPlayer();
		p.addSkill(skill("Leap"));
		assertTrue(p.hasSkillProperty(NamedProperties.canLeap));
	}

	@Test
	public void addPrayerSkillAddsToTemporarySkills() {
		RosterPlayer p = testPlayer();
		p.addTemporarySkills("STILETTO", Collections.singleton(new SkillWithValue(skill("Stab"), null)));
		assertTrue(p.has(skill("Stab")));
	}

	@Test
	public void addPrayerSkillWithValueStoresValue() {
		RosterPlayer p = testPlayer();
		p.addTemporarySkills("BAD_HABITS", Collections.singleton(new SkillWithValue(skill("Loner"), "2")));
		assertTrue(p.has(skill("Loner")));
		assertTrue(p.temporarySkillValues(skill("Loner")).contains("2"));
	}

	@Test
	public void removePrayerSkillsRemovesFromTemporary() {
		RosterPlayer p = testPlayer();
		p.addTemporarySkills("STILETTO", Collections.singleton(new SkillWithValue(skill("Stab"), null)));
		assertTrue(p.has(skill("Stab")));
		p.removeTemporarySkills("STILETTO");
		assertFalse(p.has(skill("Stab")));
		assertTrue(p.getEnhancementSources().isEmpty());
	}

	@Test
	public void removePrayerSkillsOnlyRemovesMatchingSource() {
		RosterPlayer p = testPlayer();
		p.addTemporarySkills("STILETTO", Collections.singleton(new SkillWithValue(skill("Stab"), null)));
		p.addTemporarySkills("BLESSING", Collections.singleton(new SkillWithValue(skill("Block"), null)));
		p.removeTemporarySkills("STILETTO");
		assertFalse(p.has(skill("Stab")));
		assertTrue(p.has(skill("Block")));
	}

	@Test
	public void removeSkillRemovesFromExtra() {
		RosterPlayer p = testPlayer();
		p.addSkill(skill("Dodge"));
		p.removeSkill(skill("Dodge"));
		assertFalse(p.has(skill("Dodge")));
	}

}
