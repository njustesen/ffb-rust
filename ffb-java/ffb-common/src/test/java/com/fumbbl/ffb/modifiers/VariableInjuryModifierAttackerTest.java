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
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-mechanics/src/modifiers/variable_injury_modifier_attacker.rs tests.
 * appliesToContext = context.isAttackerMode() && UtilCards.hasSkill(attacker, registeredTo).
 * EXEMPT (Rust builder-only): with_modifier_fn / with_predicate.
 */
public class VariableInjuryModifierAttackerTest {

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

	private InjuryModifierContext ctx(RosterPlayer attacker, RosterPlayer defender, boolean defenderMode) {
		InjuryModifierContext c = new InjuryModifierContext(game, null, attacker, defender, false, false, false, false);
		if (defenderMode) {
			c.setDefenderMode();
		}
		return c;
	}

	// rust: applies_false_when_defender_mode
	@Test
	public void appliesFalseWhenDefenderMode() {
		VariableInjuryModifierAttacker m = new VariableInjuryModifierAttacker("test", false);
		Skill block = skill("Block");
		m.setRegisteredTo(block);
		RosterPlayer attacker = new RosterPlayer();
		attacker.setId("a");
		attacker.addSkill(block);
		RosterPlayer defender = new RosterPlayer();
		defender.setId("d");
		assertFalse(m.appliesToContext(ctx(attacker, defender, true)));
	}

	// rust: applies_false_when_attacker_lacks_skill
	@Test
	public void appliesFalseWhenAttackerLacksSkill() {
		VariableInjuryModifierAttacker m = new VariableInjuryModifierAttacker("test", false);
		m.setRegisteredTo(skill("Block"));
		RosterPlayer attacker = new RosterPlayer();
		attacker.setId("a");
		RosterPlayer defender = new RosterPlayer();
		defender.setId("d");
		assertFalse(m.appliesToContext(ctx(attacker, defender, false)));
	}

	// rust: applies_true_when_attacker_has_registered_skill
	@Test
	public void appliesTrueWhenAttackerHasRegisteredSkill() {
		VariableInjuryModifierAttacker m = new VariableInjuryModifierAttacker("test", false);
		Skill block = skill("Block");
		m.setRegisteredTo(block);
		RosterPlayer attacker = new RosterPlayer();
		attacker.setId("a");
		attacker.addSkill(block);
		RosterPlayer defender = new RosterPlayer();
		defender.setId("d");
		assertTrue(m.appliesToContext(ctx(attacker, defender, false)));
	}

	// rust: applies_false_when_no_attacker
	@Test
	public void appliesFalseWhenNoAttacker() {
		VariableInjuryModifierAttacker m = new VariableInjuryModifierAttacker("test", false);
		m.setRegisteredTo(skill("Block"));
		RosterPlayer defender = new RosterPlayer();
		defender.setId("d");
		assertFalse(m.appliesToContext(ctx(null, defender, false)));
	}

	// rust: set_registered_to_stores_value
	@Test
	public void setRegisteredToStoresValue() {
		VariableInjuryModifierAttacker m = new VariableInjuryModifierAttacker("x", false);
		Skill dodge = skill("Dodge");
		m.setRegisteredTo(dodge);
		assertEquals(dodge, m.getRegisteredTo());
	}
}
