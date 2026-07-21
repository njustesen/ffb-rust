package com.fumbbl.ffb.factory;

import com.fumbbl.ffb.SkillCategory;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/factory/skill_category_factory.rs
 * for {@link SkillCategoryFactory}.
 */
public class SkillCategoryFactoryTest {

	@Test
	public void forNameReturnsKnownCategory() {
		assertEquals(SkillCategory.GENERAL, new SkillCategoryFactory().forName("General"));
		assertEquals(SkillCategory.AGILITY, new SkillCategoryFactory().forName("Agility"));
	}

	@Test
	public void forNameUnknownReturnsNone() {
		assertNull(new SkillCategoryFactory().forName("invalid"));
	}

	@Test
	public void forNameStrengthReturnsSome() {
		SkillCategoryFactory f = new SkillCategoryFactory();
		assertNotNull(f.forName("Strength"));
	}

	@Test
	public void forNameEmptyStringReturnsNone() {
		assertNull(new SkillCategoryFactory().forName(""));
	}
}
