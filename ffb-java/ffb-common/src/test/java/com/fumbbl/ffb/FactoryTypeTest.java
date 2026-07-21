package com.fumbbl.ffb;

import com.fumbbl.ffb.FactoryType.Factory;
import com.fumbbl.ffb.FactoryType.FactoryContext;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotEquals;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/model/factory_type.rs for {@link FactoryType}.
 */
public class FactoryTypeTest {

	@Test
	public void applicationFactoriesReturnApplicationContext() {
		assertEquals(FactoryContext.APPLICATION, Factory.NET_COMMAND_ID.context);
		assertEquals(FactoryContext.APPLICATION, Factory.GAME_STATUS.context);
	}

	@Test
	public void gameFactoriesReturnGameContext() {
		assertEquals(FactoryContext.GAME, Factory.SKILL.context);
		assertEquals(FactoryContext.GAME, Factory.ANIMATION_TYPE.context);
	}

	@Test
	public void allFiveApplicationFactoriesAreApplicationContext() {
		for (Factory f : new Factory[]{Factory.NET_COMMAND_ID, Factory.CLIENT_MODE, Factory.GAME_OPTION_ID, Factory.SERVER_STATUS, Factory.GAME_STATUS}) {
			assertEquals(FactoryContext.APPLICATION, f.context, f + " should be APPLICATION");
		}
	}

	@Test
	public void contextsAreDistinct() {
		assertNotEquals(FactoryContext.APPLICATION, FactoryContext.GAME);
	}

	@Test
	public void copySemanticsForFactory() {
		Factory a = Factory.SKILL;
		Factory b = a;
		assertEquals(a, b);
	}

}
