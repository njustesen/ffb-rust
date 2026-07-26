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
 * Mirror of ffb-rust crates/ffb-mechanics/src/modifiers/variable_injury_modifier_defender.rs tests.
 * appliesToContext = context.isDefenderMode() && UtilCards.hasSkill(defender, registeredTo).
 */
public class VariableInjuryModifierDefenderTest {

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

	private InjuryModifierContext ctx(RosterPlayer defender, boolean defenderMode) {
		InjuryModifierContext c = new InjuryModifierContext(game, null, null, defender, false, false, false, false);
		if (defenderMode) {
			c.setDefenderMode();
		}
		return c;
	}

	// rust: applies_false_when_attacker_mode
	@Test
	public void appliesFalseWhenAttackerMode() {
		VariableInjuryModifierDefender m = new VariableInjuryModifierDefender("test", false);
		Skill dodge = skill("Dodge");
		m.setRegisteredTo(dodge);
		RosterPlayer defender = new RosterPlayer();
		defender.setId("d");
		defender.addSkill(dodge);
		assertFalse(m.appliesToContext(ctx(defender, false)));
	}

	// rust: applies_false_when_defender_lacks_skill
	@Test
	public void appliesFalseWhenDefenderLacksSkill() {
		VariableInjuryModifierDefender m = new VariableInjuryModifierDefender("test", false);
		m.setRegisteredTo(skill("Dodge"));
		RosterPlayer defender = new RosterPlayer();
		defender.setId("d");
		assertFalse(m.appliesToContext(ctx(defender, true)));
	}

	// rust: applies_true_when_defender_has_registered_skill
	@Test
	public void appliesTrueWhenDefenderHasRegisteredSkill() {
		VariableInjuryModifierDefender m = new VariableInjuryModifierDefender("test", false);
		Skill dodge = skill("Dodge");
		m.setRegisteredTo(dodge);
		RosterPlayer defender = new RosterPlayer();
		defender.setId("d");
		defender.addSkill(dodge);
		assertTrue(m.appliesToContext(ctx(defender, true)));
	}

	// rust: set_registered_to_stores_value
	@Test
	public void setRegisteredToStoresValue() {
		VariableInjuryModifierDefender m = new VariableInjuryModifierDefender("x", false);
		Skill dodge = skill("Dodge");
		m.setRegisteredTo(dodge);
		assertEquals(dodge, m.getRegisteredTo());
	}

	// rust: set_registered_to_then_clear_to_none
	@Test
	public void setRegisteredToThenClearToNone() {
		VariableInjuryModifierDefender m = new VariableInjuryModifierDefender("x", false);
		m.setRegisteredTo(skill("Dodge"));
		m.setRegisteredTo(null);
		assertNull(m.getRegisteredTo());
	}
}
