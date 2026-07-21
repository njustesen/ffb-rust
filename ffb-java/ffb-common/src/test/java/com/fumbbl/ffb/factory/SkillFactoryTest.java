package com.fumbbl.ffb.factory;

import com.fumbbl.ffb.FactoryType.Factory;
import com.fumbbl.ffb.model.skill.Skill;
import com.fumbbl.ffb.net.NetCommandTestUtil;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/factory/skill_factory.rs
 * for {@link SkillFactory}.
 * <p>
 * Only the case-insensitive {@code forName} lookups are ported. The Rust {@code for_class_name}
 * tests have no Java counterpart (the Java factory keys skills by display name, not class name),
 * and the skill-count tests are intentionally skipped because the Java skill table differs in
 * size from the Rust table.
 */
public class SkillFactoryTest {

	private static SkillFactory factory() {
		return NetCommandTestUtil.gameSource().getFactory(Factory.SKILL);
	}

	@Test
	public void forNameBallAndChainAlias() {
		SkillFactory factory = factory();
		assertNotNull(factory.forName("Ball & Chain"));
		assertNotNull(factory.forName("ball & chain"));
		assertNotNull(factory.forName("Ball &amp; Chain"));
	}

	@Test
	public void forNameBlockLowercase() {
		Skill skill = factory().forName("block");
		assertNotNull(skill);
	}

	@Test
	public void forNameUnknownReturnsNone() {
		assertNull(factory().forName("no such skill"));
	}
}
