package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_journeymen.rs tests.
 *
 * The Rust struct exposes incremental add_slot/add_position_id; the Java class only accepts
 * slots and position ids through the array constructor, so those values are supplied up front.
 */
public class ClientCommandJourneymenTest {

	@Test
	public void addSlotsAndPositions() {
		ClientCommandJourneymen cmd = new ClientCommandJourneymen(new String[] { "pos_lineman" }, new int[] { 1, 2 });
		assertArrayEquals(new int[] { 1, 2 }, cmd.getSlots());
		assertArrayEquals(new String[] { "pos_lineman" }, cmd.getPositionIds());
	}

	@Test
	public void defaultEmptyVecs() {
		ClientCommandJourneymen cmd = new ClientCommandJourneymen();
		assertEquals(0, cmd.getSlots().length);
		assertEquals(0, cmd.getPositionIds().length);
	}

	@Test
	public void slotsCanHoldMultiple() {
		ClientCommandJourneymen cmd = new ClientCommandJourneymen(new String[] {}, new int[] { 5, 6 });
		assertEquals(2, cmd.getSlots().length);
	}

	@Test
	public void getIdIsClientJourneymen() {
		assertEquals(NetCommandId.CLIENT_JOURNEYMEN, new ClientCommandJourneymen().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndSlots() {
		ClientCommandJourneymen cmd = new ClientCommandJourneymen(new String[] { "pos_1" }, new int[] { 3 });
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientJourneymen", json.get("netCommandId").asString());
		assertEquals(1, json.get("slots").asArray().size());
		assertEquals(3, json.get("slots").asArray().get(0).asInt());
		assertEquals("pos_1", json.get("positionIds").asArray().get(0).asString());
	}

	@Test
	public void roundTripWithSlotsAndEntropy() {
		ClientCommandJourneymen cmd = new ClientCommandJourneymen(new String[] { "pos_lineman" }, new int[] { 1, 2 });
		cmd.setEntropy((byte) 2);
		JsonObject json = cmd.toJsonValue();
		ClientCommandJourneymen restored = new ClientCommandJourneymen().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals((byte) 2, restored.getEntropy());
		assertArrayEquals(new int[] { 1, 2 }, restored.getSlots());
		assertArrayEquals(new String[] { "pos_lineman" }, restored.getPositionIds());
	}

	@Test
	public void roundTripWithEmptyVecs() {
		ClientCommandJourneymen cmd = new ClientCommandJourneymen();
		JsonObject json = cmd.toJsonValue();
		ClientCommandJourneymen restored = new ClientCommandJourneymen().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals(0, restored.getSlots().length);
		assertEquals(0, restored.getPositionIds().length);
	}
}
