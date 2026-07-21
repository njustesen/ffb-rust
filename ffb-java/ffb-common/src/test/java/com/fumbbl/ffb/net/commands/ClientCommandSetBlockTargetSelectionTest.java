package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.model.BlockKind;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_set_block_target_selection.rs tests.
 *
 * Note: Java {@code toJsonValue()} calls {@code kind.name()} unconditionally, so it
 * NPEs when {@code kind} is null. The Rust {@code round_trip_default} test (default
 * command with no kind) is therefore inexpressible in Java and is SKIPPED.
 */
public class ClientCommandSetBlockTargetSelectionTest {

	@Test
	public void fieldsStored() {
		ClientCommandSetBlockTargetSelection cmd = new ClientCommandSetBlockTargetSelection("p1", BlockKind.BLOCK);
		assertEquals("p1", cmd.getPlayerId());
		assertEquals(BlockKind.BLOCK, cmd.getKind());
	}

	@Test
	public void defaultIsEmpty() {
		ClientCommandSetBlockTargetSelection cmd = new ClientCommandSetBlockTargetSelection();
		assertNull(cmd.getPlayerId());
		assertNull(cmd.getKind());
	}

	@Test
	public void stabKindStored() {
		ClientCommandSetBlockTargetSelection cmd = new ClientCommandSetBlockTargetSelection("p2", BlockKind.STAB);
		assertEquals(BlockKind.STAB, cmd.getKind());
		assertEquals("p2", cmd.getPlayerId());
	}

	@Test
	public void getIdIsClientSetBlockTargetSelection() {
		assertEquals(NetCommandId.CLIENT_SET_BLOCK_TARGET_SELECTION, new ClientCommandSetBlockTargetSelection().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndBlockKind() {
		ClientCommandSetBlockTargetSelection cmd = new ClientCommandSetBlockTargetSelection("p1", BlockKind.STAB);
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientSetBlockTargetSelection", json.get("netCommandId").asString());
		assertEquals("STAB", json.get("blockKind").asString());
		assertEquals("p1", json.get("playerId").asString());
	}

	@Test
	public void roundTripPopulated() {
		ClientCommandSetBlockTargetSelection cmd = new ClientCommandSetBlockTargetSelection("p2", BlockKind.CHAINSAW);
		cmd.setEntropy((byte) 4);
		JsonObject json = cmd.toJsonValue();
		ClientCommandSetBlockTargetSelection restored = (ClientCommandSetBlockTargetSelection) new ClientCommandSetBlockTargetSelection().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals("p2", restored.getPlayerId());
		assertEquals(BlockKind.CHAINSAW, restored.getKind());
		assertEquals((byte) 4, restored.getEntropy());
	}
}
