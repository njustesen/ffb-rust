package com.fumbbl.ffb.util;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.factory.IFactorySource;
import com.fumbbl.ffb.factory.SkillFactory;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.RosterPlayer;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.model.skill.Skill;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-model/src/util/util_cards.rs tests.
 * Real skills come from NetCommandTestUtil.gameSource()'s SkillFactory (whose factories are loaded,
 * unlike an applicationSource Game). The Rust hasUnused tests use Dodge's generic first property; a
 * concrete Wrestle + canTakeDownPlayersWithHimOnBothDown pairing is used here (same has/used/absent
 * behavior).
 */
public class UtilCardsTest {

	private Game game;

	@BeforeEach
	void setUp() {
		IFactorySource app = NetCommandTestUtil.applicationSource();
		game = new Game(app, app.getFactoryManager());
	}

	private Skill skill(String name) {
		SkillFactory factory = NetCommandTestUtil.gameSource().getFactory(FactoryType.Factory.SKILL);
		return factory.forName(name);
	}

	private RosterPlayer playerWithSkill(String skillName) {
		RosterPlayer p = new RosterPlayer();
		p.setId("p1");
		p.addSkill(skill(skillName));
		return p;
	}

	// rust: has_unused_skill_with_property_true_when_skill_present_and_unused
	@Test
	public void hasUnusedSkillWithPropertyTrueWhenSkillPresentAndUnused() {
		RosterPlayer p = playerWithSkill("Wrestle");
		assertTrue(UtilCards.hasUnusedSkillWithProperty(p, NamedProperties.canTakeDownPlayersWithHimOnBothDown));
	}

	// rust: has_unused_skill_with_property_false_when_skill_used
	@Test
	public void hasUnusedSkillWithPropertyFalseWhenSkillUsed() {
		RosterPlayer p = playerWithSkill("Wrestle");
		p.markUsed(skill("Wrestle"), game);
		assertFalse(UtilCards.hasUnusedSkillWithProperty(p, NamedProperties.canTakeDownPlayersWithHimOnBothDown));
	}

	// rust: has_unused_skill_with_property_false_when_no_skill
	@Test
	public void hasUnusedSkillWithPropertyFalseWhenNoSkill() {
		RosterPlayer p = playerWithSkill("Block");
		assertFalse(UtilCards.hasUnusedSkillWithProperty(p, NamedProperties.canTakeDownPlayersWithHimOnBothDown));
	}

	// rust: get_unused_skill_with_property_returns_skill_id_when_present_and_none_when_absent
	@Test
	public void getUnusedSkillWithPropertyPresentAndAbsent() {
		RosterPlayer p = playerWithSkill("Wrestle");
		assertTrue(UtilCards.getUnusedSkillWithProperty(p, NamedProperties.canTakeDownPlayersWithHimOnBothDown).isPresent());
		RosterPlayer p2 = playerWithSkill("Block");
		assertTrue(UtilCards.getUnusedSkillWithProperty(p2, NamedProperties.canTakeDownPlayersWithHimOnBothDown).isEmpty());
	}

	// rust: has_skill_to_cancel_property_true_for_tackle_cancelling_dodge_reroll
	@Test
	public void hasSkillToCancelPropertyTrueForTackleCancellingDodgeReroll() {
		RosterPlayer p = playerWithSkill("Tackle");
		assertTrue(UtilCards.hasSkillToCancelProperty(p, NamedProperties.canRerollDodge));
		RosterPlayer p2 = playerWithSkill("Block");
		assertFalse(UtilCards.hasSkillToCancelProperty(p2, NamedProperties.canRerollDodge));
	}
}
