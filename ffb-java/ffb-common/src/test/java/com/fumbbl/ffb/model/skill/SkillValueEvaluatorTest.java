package com.fumbbl.ffb.model.skill;

import org.junit.jupiter.api.Test;

import java.util.Arrays;
import java.util.HashSet;
import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/model/skill/skill_value_evaluator.rs for {@link SkillValueEvaluator}.
 */
public class SkillValueEvaluatorTest {

	private static Set<String> setOf(String... vals) {
		return new HashSet<>(Arrays.asList(vals));
	}

	@Test
	public void modifierReturnsMax() {
		assertEquals(Integer.valueOf(3), SkillValueEvaluator.MODIFIER.intValue(setOf("1", "3", "2")));
	}

	@Test
	public void rollReturnsMin() {
		assertEquals(Integer.valueOf(2), SkillValueEvaluator.ROLL.intValue(setOf("4", "2", "3")));
	}

	@Test
	public void emptySetModifierReturnsNone() {
		assertNull(SkillValueEvaluator.MODIFIER.intValue(setOf()));
	}

	@Test
	public void emptySetRollReturnsNone() {
		assertNull(SkillValueEvaluator.ROLL.intValue(setOf()));
	}

	@Test
	public void nonNumericStringsAreIgnored() {
		assertEquals(Integer.valueOf(5), SkillValueEvaluator.MODIFIER.intValue(setOf("foo", "bar", "5")));
	}
}
