package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/server_command_replay_control.rs tests.
 */
public class ServerCommandReplayControlTest {

	@Test
	public void coachStored() {
		ServerCommandReplayControl cmd = new ServerCommandReplayControl("Alice");
		assertEquals("Alice", cmd.getCoach());
	}

	@Test
	public void defaultEmpty() {
		ServerCommandReplayControl cmd = new ServerCommandReplayControl();
		// Java leaves coach null (Rust default() is empty string).
		assertNull(cmd.getCoach());
	}

	@Test
	public void getIdIsServerReplayControl() {
		assertEquals(NetCommandId.SERVER_REPLAY_CONTROL, new ServerCommandReplayControl().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndCoach() {
		ServerCommandReplayControl cmd = new ServerCommandReplayControl("Alice");
		JsonObject json = cmd.toJsonValue();
		assertEquals("serverReplayControl", json.get("netCommandId").asString());
		assertEquals("Alice", json.get("coach").asString());
	}

	@Test
	public void roundTripWithData() {
		ServerCommandReplayControl cmd = new ServerCommandReplayControl("Bob");
		JsonObject json = cmd.toJsonValue();
		ServerCommandReplayControl restored = new ServerCommandReplayControl().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals("Bob", restored.getCoach());
	}

	@Test
	public void roundTripWithDefault() {
		ServerCommandReplayControl cmd = new ServerCommandReplayControl();
		JsonObject json = cmd.toJsonValue();
		ServerCommandReplayControl restored = new ServerCommandReplayControl().initFrom(NetCommandTestUtil.gameSource(), json);
		// Java restores null (Rust default() is empty string).
		assertNull(restored.getCoach());
	}
}
