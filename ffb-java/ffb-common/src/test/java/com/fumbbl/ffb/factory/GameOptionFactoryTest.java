package com.fumbbl.ffb.factory;

import com.fumbbl.ffb.option.GameOptionId;
import com.fumbbl.ffb.option.GameOptionString;
import com.fumbbl.ffb.option.IGameOption;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/factory/game_option_factory.rs
 * for {@link GameOptionFactory}.
 */
public class GameOptionFactoryTest {

	@Test
	public void createBooleanOptionHasDefault() {
		GameOptionFactory factory = new GameOptionFactory();
		IGameOption opt = factory.createGameOption(GameOptionId.ALLOW_CONCESSIONS);
		assertEquals("true", opt.getValueAsString());
		assertEquals("allowConcessions", opt.getId().getName());
	}

	@Test
	public void createIntOptionHasDefault() {
		GameOptionFactory factory = new GameOptionFactory();
		IGameOption opt = factory.createGameOption(GameOptionId.TURNTIME);
		assertEquals("240", opt.getValueAsString());
	}

	@Test
	public void createStringOptionWithDefault() {
		GameOptionFactory factory = new GameOptionFactory();
		IGameOption opt = factory.createGameOption(GameOptionId.RULESVERSION);
		assertEquals("BB2016", opt.getValueAsString());
	}

	@Test
	public void createMaxPlayersOnFieldDefault() {
		GameOptionFactory factory = new GameOptionFactory();
		IGameOption opt = factory.createGameOption(GameOptionId.MAX_PLAYERS_ON_FIELD);
		assertEquals("11", opt.getValueAsString());
	}

	@Test
	public void everyVariantProducesAnOption() {
		GameOptionFactory factory = new GameOptionFactory();
		for (GameOptionId id : GameOptionId.values()) {
			IGameOption opt = factory.createGameOption(id);
			assertNotNull(opt, "expected an option for " + id);
			assertEquals(id.getName(), opt.getId().getName());
		}
	}

	@Test
	public void chainsawTurnoverUsesKickbackDefault() {
		GameOptionFactory factory = new GameOptionFactory();
		IGameOption opt = factory.createGameOption(GameOptionId.CHAINSAW_TURNOVER);
		assertEquals(GameOptionString.CHAINSAW_TURNOVER_KICKBACK, opt.getValueAsString());
	}

	@Test
	public void booleanOptionDefaultFalse() {
		GameOptionFactory factory = new GameOptionFactory();
		IGameOption opt = factory.createGameOption(GameOptionId.TEST_MODE);
		assertEquals("false", opt.getValueAsString());
	}
}
