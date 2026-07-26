package com.fumbbl.ffb.modifiers;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.factory.IFactorySource;
import com.fumbbl.ffb.factory.SkillFactory;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.RosterPlayer;
import com.fumbbl.ffb.model.skill.Skill;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-mechanics/src/modifiers/static_injury_modifier_defender.rs tests.
 * appliesToContext = UtilCards.hasSkill(context.getDefender(), registeredTo).
 * <p>
 * EXEMPT: the Rust applies_true_when_no_registered_to test asserts true for a null registered_to —
 * same injury-factory ARCHITECTURE divergence documented in StaticInjuryModifierAttackerTest (Java
 * always registers to a skill; UtilCards.hasSkill(defender, null) is false). The skill-presence
 * branches below map 1:1.
 */
public class StaticInjuryModifierDefenderTest {

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

	private InjuryModifierContext ctx(RosterPlayer defender) {
		return new InjuryModifierContext(game, null, null, defender, false, false, false, false);
	}

	// rust: applies_false_when_defender_lacks_skill
	@Test
	public void appliesFalseWhenDefenderLacksSkill() {
		StaticInjuryModifierDefender m = new StaticInjuryModifierDefender("test", 1, false);
		m.setRegisteredTo(skill("Dodge"));
		RosterPlayer defender = new RosterPlayer();
		defender.setId("d");
		assertFalse(m.appliesToContext(ctx(defender)));
	}

	// rust: applies_true_when_defender_has_registered_skill
	@Test
	public void appliesTrueWhenDefenderHasRegisteredSkill() {
		StaticInjuryModifierDefender m = new StaticInjuryModifierDefender("test", 1, false);
		Skill dodge = skill("Dodge");
		m.setRegisteredTo(dodge);
		RosterPlayer defender = new RosterPlayer();
		defender.setId("d");
		defender.addSkill(dodge);
		assertTrue(m.appliesToContext(ctx(defender)));
	}

	// rust: set_registered_to_stores_value
	@Test
	public void setRegisteredToStoresValue() {
		StaticInjuryModifierDefender m = new StaticInjuryModifierDefender("x", 0, false);
		Skill dodge = skill("Dodge");
		m.setRegisteredTo(dodge);
		assertEquals(dodge, m.getRegisteredTo());
	}

	// rust: set_registered_to_then_clear_to_none
	@Test
	public void setRegisteredToThenClearToNone() {
		StaticInjuryModifierDefender m = new StaticInjuryModifierDefender("x", 0, false);
		m.setRegisteredTo(skill("Dodge"));
		m.setRegisteredTo(null);
		assertNull(m.getRegisteredTo());
	}
}
