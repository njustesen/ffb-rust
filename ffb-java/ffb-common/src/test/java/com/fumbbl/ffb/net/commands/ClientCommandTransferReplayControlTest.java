package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_transfer_replay_control.rs tests.
 */
public class ClientCommandTransferReplayControlTest {

	@Test
	public void defaultHasNoCoach() {
		ClientCommandTransferReplayControl cmd = new ClientCommandTransferReplayControl();
		assertNull(cmd.getCoach());
	}

	@Test
	public void withCoachStoresValue() {
		ClientCommandTransferReplayControl cmd = new ClientCommandTransferReplayControl("coach-abc");
		assertEquals("coach-abc", cmd.getCoach());
	}

	@Test
	public void getIdIsClientTransferReplayControl() {
		assertEquals(NetCommandId.CLIENT_TRANSFER_REPLAY_CONTROL, new ClientCommandTransferReplayControl().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndCoach() {
		ClientCommandTransferReplayControl cmd = new ClientCommandTransferReplayControl("coach-1");
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientTransferReplayControl", json.get("netCommandId").asString());
		assertEquals("coach-1", json.get("coach").asString());
	}

	@Test
	public void roundTripWithCoachAndEntropy() {
		ClientCommandTransferReplayControl cmd = new ClientCommandTransferReplayControl("coach-2");
		cmd.setEntropy((byte) 6);
		JsonObject json = cmd.toJsonValue();
		ClientCommandTransferReplayControl restored = new ClientCommandTransferReplayControl().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals((byte) 6, restored.getEntropy());
		assertEquals("coach-2", restored.getCoach());
	}

	@Test
	public void roundTripWithNoCoach() {
		ClientCommandTransferReplayControl cmd = new ClientCommandTransferReplayControl();
		JsonObject json = cmd.toJsonValue();
		ClientCommandTransferReplayControl restored = new ClientCommandTransferReplayControl().initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getCoach());
	}
}
