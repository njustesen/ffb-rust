package com.fumbbl.ffb.net;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/socket_change_request.rs tests.
 * Java SocketChangeRequest holds an actual SocketChannel (not an integer handle)
 * and does NOT override equals(); the Rust `equality` test therefore has no Java
 * analogue and is intentionally not ported (see report). new_sets_fields is
 * expressed against getType()/getOps()/getSocketChannel().
 */
public class SocketChangeRequestTest {

	@Test
	public void constantsMatchJava() {
		assertEquals(1, SocketChangeRequest.REGISTER);
		assertEquals(2, SocketChangeRequest.CHANGEOPS);
	}

	@Test
	public void newSetsFields() {
		SocketChangeRequest req = new SocketChangeRequest(null, SocketChangeRequest.REGISTER, 1);
		assertNull(req.getSocketChannel());
		assertEquals(SocketChangeRequest.REGISTER, req.getType());
		assertEquals(1, req.getOps());
	}
}
