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
 * Mirror of ffb-rust crates/ffb-mechanics/src/modifiers/static_injury_modifier_attacker.rs tests.
 * appliesToContext = UtilCards.hasSkill(context.getAttacker(), registeredTo).
 * <p>
 * EXEMPT (no faithful Java twin): the Rust applies_true_when_no_registered_to test asserts true for
 * a null registered_to — that reflects the Rust injury-factory ARCHITECTURE, which pre-filters by
 * skill before constructing the modifier (so registered_to stays None = "already qualified"). Java
 * instead registers every such modifier to its owning skill, so registeredTo is never null in
 * practice; UtilCards.hasSkill(attacker, null) returns false. Also EXEMPT: with_predicate override
 * (Rust builder-only). The skill-presence branches below map 1:1.
 */
public class StaticInjuryModifierAttackerTest {

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

	private InjuryModifierContext ctx(RosterPlayer attacker, RosterPlayer defender) {
		return new InjuryModifierContext(game, null, attacker, defender, false, false, false, false);
	}

	// rust: applies_false_when_no_attacker
	@Test
	public void appliesFalseWhenNoAttacker() {
		StaticInjuryModifierAttacker m = new StaticInjuryModifierAttacker("test", 1, false);
		m.setRegisteredTo(skill("Block"));
		RosterPlayer defender = new RosterPlayer();
		defender.setId("d");
		assertFalse(m.appliesToContext(ctx(null, defender)));
	}

	// rust: applies_false_when_attacker_lacks_skill
	@Test
	public void appliesFalseWhenAttackerLacksSkill() {
		StaticInjuryModifierAttacker m = new StaticInjuryModifierAttacker("test", 1, false);
		m.setRegisteredTo(skill("Block"));
		RosterPlayer attacker = new RosterPlayer();
		attacker.setId("a");
		RosterPlayer defender = new RosterPlayer();
		defender.setId("d");
		assertFalse(m.appliesToContext(ctx(attacker, defender)));
	}

	// rust: applies_true_when_attacker_has_registered_skill
	@Test
	public void appliesTrueWhenAttackerHasRegisteredSkill() {
		StaticInjuryModifierAttacker m = new StaticInjuryModifierAttacker("test", 1, false);
		Skill block = skill("Block");
		m.setRegisteredTo(block);
		RosterPlayer attacker = new RosterPlayer();
		attacker.setId("a");
		attacker.addSkill(block);
		RosterPlayer defender = new RosterPlayer();
		defender.setId("d");
		assertTrue(m.appliesToContext(ctx(attacker, defender)));
	}

	// rust: set_registered_to_stores_value
	@Test
	public void setRegisteredToStoresValue() {
		StaticInjuryModifierAttacker m = new StaticInjuryModifierAttacker("x", 0, false);
		Skill dodge = skill("Dodge");
		m.setRegisteredTo(dodge);
		assertEquals(dodge, m.getRegisteredTo());
	}
}
