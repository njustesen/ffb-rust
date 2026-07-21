package com.fumbbl.ffb.net.commands;

import com.fumbbl.ffb.FactoryType.FactoryContext;
import com.fumbbl.ffb.net.NetCommand;
import com.fumbbl.ffb.net.NetCommandId;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertDoesNotThrow;
import static org.junit.jupiter.api.Assertions.assertThrows;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/util_net_command.rs tests.
 * The Rust helper returns Result; the Java validateCommandId throws
 * IllegalStateException on mismatch and IllegalArgumentException on a null
 * command, so the Ok/Err assertions map onto does-not-throw / throws.
 */
public class UtilNetCommandTest {

	private static NetCommand commandWithId(final NetCommandId id) {
		return new NetCommand() {
			public NetCommandId getId() {
				return id;
			}

			public FactoryContext getContext() {
				return FactoryContext.GAME;
			}
		};
	}

	@Test
	public void matchingIdsOk() {
		assertDoesNotThrow(() ->
			UtilNetCommand.validateCommandId(commandWithId(NetCommandId.CLIENT_JOIN), NetCommandId.CLIENT_JOIN));
	}

	@Test
	public void mismatchedIdsErr() {
		assertThrows(IllegalStateException.class, () ->
			UtilNetCommand.validateCommandId(commandWithId(NetCommandId.CLIENT_JOIN), NetCommandId.CLIENT_TALK));
	}

	@Test
	public void noneReceivedIdErr() {
		assertThrows(IllegalStateException.class, () ->
			UtilNetCommand.validateCommandId(commandWithId(NetCommandId.CLIENT_JOIN), null));
	}
}
