package com.fumbbl.ffb.net;

import com.fumbbl.ffb.FactoryType.FactoryContext;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/net_command_log.rs tests.
 * Java NetCommandLog exposes add() and getCommands() (array); there is no
 * len()/isEmpty(), so the Rust size assertions map onto getCommands().length.
 */
public class NetCommandLogTest {

	private static class Dummy extends NetCommand {
		public NetCommandId getId() {
			return NetCommandId.CLIENT_JOIN;
		}

		public FactoryContext getContext() {
			return FactoryContext.GAME;
		}
	}

	@Test
	public void newLogIsEmpty() {
		NetCommandLog log = new NetCommandLog();
		assertEquals(0, log.getCommands().length);
	}

	@Test
	public void addIncrementsLen() {
		NetCommandLog log = new NetCommandLog();
		log.add(new Dummy());
		log.add(new Dummy());
		assertEquals(2, log.getCommands().length);
	}

	@Test
	public void commandsReturnsSlice() {
		NetCommandLog log = new NetCommandLog();
		log.add(new Dummy());
		assertEquals(1, log.getCommands().length);
	}
}
