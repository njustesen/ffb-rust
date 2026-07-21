package com.fumbbl.ffb.net;

import com.eclipsesource.json.JsonValue;
import com.fumbbl.ffb.FactoryType.FactoryContext;
import com.fumbbl.ffb.factory.IFactorySource;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/i_net_command_handler.rs tests.
 */
public class INetCommandHandlerTest {

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

	private static class Counter implements INetCommandHandler {
		int count;

		public void handleCommand(NetCommand pNetCommand) {
			count++;
		}
	}

	@Test
	public void handleCommandIncrementsCounter() {
		Counter h = new Counter();
		h.handleCommand(new Dummy());
		assertEquals(1, h.count);
	}

	@Test
	public void handleCommandCalledMultipleTimes() {
		Counter h = new Counter();
		Dummy cmd = new Dummy();
		h.handleCommand(cmd);
		h.handleCommand(cmd);
		assertEquals(2, h.count);
	}
}
