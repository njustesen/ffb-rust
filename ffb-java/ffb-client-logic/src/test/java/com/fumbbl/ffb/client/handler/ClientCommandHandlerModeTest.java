package com.fumbbl.ffb.client.handler;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotEquals;

/**
 * Mirrors Rust {@code client_command_handler_mode.rs} tests. The Java type is a plain enum
 * (no Rust-only helper methods), so the port keeps only the directly translatable assertions.
 */
class ClientCommandHandlerModeTest {

	@Test
	void variantsAreDistinct() {
		assertNotEquals(ClientCommandHandlerMode.PLAYING, ClientCommandHandlerMode.REPLAYING);
		assertNotEquals(ClientCommandHandlerMode.INITIALIZING, ClientCommandHandlerMode.QUEUING);
	}

	@Test
	void allFourVariantsExist() {
		ClientCommandHandlerMode[] all = ClientCommandHandlerMode.values();
		assertEquals(4, all.length);
	}
}
