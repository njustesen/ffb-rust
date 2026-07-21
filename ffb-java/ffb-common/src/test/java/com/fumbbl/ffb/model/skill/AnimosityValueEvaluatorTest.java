package com.fumbbl.ffb.model.skill;

import com.fumbbl.ffb.skill.bb2020.Animosity;

import org.junit.jupiter.api.Test;

import java.util.HashSet;
import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/model/skill/animosity_value_evaluator.rs for {@link AnimosityValueEvaluator}.
 */
public class AnimosityValueEvaluatorTest {

	@Test
	public void allValueReturnsAll() {
		AnimosityValueEvaluator evaluator = new Animosity().evaluator();
		assertEquals("all", evaluator.allValue());
	}

	@Test
	public void intValueReturnsNoneForEmpty() {
		AnimosityValueEvaluator evaluator = new Animosity().evaluator();
		Set<String> vals = new HashSet<>();
		assertNull(evaluator.intValue(vals));
	}
}
