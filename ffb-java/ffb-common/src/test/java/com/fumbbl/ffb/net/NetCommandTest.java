package com.fumbbl.ffb.net;

import com.eclipsesource.json.JsonValue;
import com.fumbbl.ffb.FactoryType.FactoryContext;
import com.fumbbl.ffb.factory.IFactorySource;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/net_command.rs tests.
 * NetCommand is abstract with abstract getId()/getContext() and a default
 * isInternal() == false. Small concrete subclasses reproduce the Rust Dummy /
 * AppScoped types.
 */
public class NetCommandTest {

	private static class Dummy extends NetCommand {
		public NetCommandId getId() {
			return NetCommandId.CLIENT_JOIN;
		}

		public FactoryContext getContext() {
			return FactoryContext.GAME;
		}

		public JsonValue toJsonValue() {
			return null;
		}

		public Object initFrom(IFactorySource source, JsonValue jsonValue) {
			return this;
		}
	}

	private static class AppScoped extends NetCommand {
		public NetCommandId getId() {
			return NetCommandId.SERVER_GAME_TIME;
		}

		public FactoryContext getContext() {
			return FactoryContext.APPLICATION;
		}

		public JsonValue toJsonValue() {
			return null;
		}

		public Object initFrom(IFactorySource source, JsonValue jsonValue) {
			return this;
		}
	}

	@Test
	public void defaultContextIsGame() {
		assertEquals(FactoryContext.GAME, new Dummy().getContext());
	}

	@Test
	public void defaultIsInternalFalse() {
		assertFalse(new Dummy().isInternal());
	}

	@Test
	public void getIdReturnsOverriddenValue() {
		assertEquals(NetCommandId.CLIENT_JOIN, new Dummy().getId());
	}

	@Test
	public void contextCanBeOverridden() {
		assertEquals(FactoryContext.APPLICATION, new AppScoped().getContext());
	}
}
