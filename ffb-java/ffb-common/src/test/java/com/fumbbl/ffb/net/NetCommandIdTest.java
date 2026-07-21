package com.fumbbl.ffb.net;

import com.fumbbl.ffb.factory.application.NetCommandIdFactory;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/net_command_id.rs tests.
 * The Rust round-trip serializes the enum to its wire name and back; the Java
 * analogue is getName() plus NetCommandIdFactory.forName() (the resolver used by
 * JsonEnumWithNameOption during deserialization).
 */
public class NetCommandIdTest {

	@Test
	public void clientJoinNameSerializes() {
		assertEquals("clientJoin", NetCommandId.CLIENT_JOIN.getName());
	}

	@Test
	public void serverGameStateRoundTrip() {
		NetCommandId id = NetCommandId.SERVER_GAME_STATE;
		NetCommandId back = new NetCommandIdFactory().forName(id.getName());
		assertEquals(id, back);
	}
}
