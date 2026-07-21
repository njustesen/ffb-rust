package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.model.BlockKind;
import com.fumbbl.ffb.model.BlockTarget;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.Collections;
import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_synchronous_multi_block.rs tests.
 */
public class ClientCommandSynchronousMultiBlockTest {

	@Test
	public void targetsStored() {
		List<BlockTarget> targets = Collections.singletonList(new BlockTarget("p1", BlockKind.BLOCK, null));
		ClientCommandSynchronousMultiBlock cmd = new ClientCommandSynchronousMultiBlock(targets);
		assertEquals(1, cmd.getSelectedTargets().size());
	}

	@Test
	public void defaultIsEmpty() {
		ClientCommandSynchronousMultiBlock cmd = new ClientCommandSynchronousMultiBlock();
		assertTrue(cmd.getSelectedTargets().isEmpty());
	}

	@Test
	public void getIdIsClientSynchronousMultiBlock() {
		assertEquals(NetCommandId.CLIENT_SYNCHRONOUS_MULTI_BLOCK, new ClientCommandSynchronousMultiBlock().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndBlockKind() {
		List<BlockTarget> targets = Collections.singletonList(new BlockTarget("p1", BlockKind.STAB, null));
		ClientCommandSynchronousMultiBlock cmd = new ClientCommandSynchronousMultiBlock(targets);
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientSynchronousMultiBlock", json.get("netCommandId").asString());
		assertEquals("STAB", json.get("selectedBlockTargets").asArray().get(0).asObject().get("blockKind").asString());
	}

	@Test
	public void roundTripWithTargetsAndEntropy() {
		List<BlockTarget> targets = Arrays.asList(
			new BlockTarget("p1", BlockKind.BLOCK, null),
			new BlockTarget("p2", BlockKind.CHAINSAW, null));
		ClientCommandSynchronousMultiBlock cmd = new ClientCommandSynchronousMultiBlock(targets);
		cmd.setEntropy((byte) 3);
		JsonObject json = cmd.toJsonValue();
		ClientCommandSynchronousMultiBlock restored = (ClientCommandSynchronousMultiBlock)
			new ClientCommandSynchronousMultiBlock().initFrom(NetCommandTestUtil.gameSource(), json);
		assertTrue(restored.hasEntropy());
		assertEquals((byte) 3, restored.getEntropy());
		assertEquals(2, restored.getSelectedTargets().size());
		assertEquals("p1", restored.getSelectedTargets().get(0).getPlayerId());
		assertEquals(BlockKind.CHAINSAW, restored.getSelectedTargets().get(1).getKind());
	}

	@Test
	public void fromJsonCapsAtTwoTargets() {
		List<BlockTarget> targets = new ArrayList<>(Arrays.asList(
			new BlockTarget("p1", BlockKind.BLOCK, null),
			new BlockTarget("p2", BlockKind.STAB, null),
			new BlockTarget("p3", BlockKind.VOMIT, null)));
		ClientCommandSynchronousMultiBlock cmd = new ClientCommandSynchronousMultiBlock(targets);
		JsonObject json = cmd.toJsonValue();
		ClientCommandSynchronousMultiBlock restored = (ClientCommandSynchronousMultiBlock)
			new ClientCommandSynchronousMultiBlock().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals(2, restored.getSelectedTargets().size());
	}

	@Test
	public void roundTripWithNoTargets() {
		ClientCommandSynchronousMultiBlock cmd = new ClientCommandSynchronousMultiBlock();
		JsonObject json = cmd.toJsonValue();
		ClientCommandSynchronousMultiBlock restored = (ClientCommandSynchronousMultiBlock)
			new ClientCommandSynchronousMultiBlock().initFrom(NetCommandTestUtil.gameSource(), json);
		assertTrue(restored.getSelectedTargets().isEmpty());
	}
}
