package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/server_command_replay.rs tests.
 * The Rust struct only stores the shared ServerCommand base shape; the Java class holds
 * concrete polymorphic ServerCommand subclasses, so ServerCommandRemovePlayer instances are
 * used as the batched replay commands here.
 */
public class ServerCommandReplayTest {

	private ServerCommandRemovePlayer replayCommand(int nr) {
		ServerCommandRemovePlayer cmd = new ServerCommandRemovePlayer("p" + nr);
		cmd.setCommandNr(nr);
		return cmd;
	}

	@Test
	public void newIsEmpty() {
		ServerCommandReplay r = new ServerCommandReplay();
		assertEquals(0, r.getNrOfCommands());
		assertFalse(r.isLastCommand());
	}

	@Test
	public void addIncrementsCount() {
		ServerCommandReplay r = new ServerCommandReplay();
		r.add(replayCommand(1));
		r.add(replayCommand(2));
		assertEquals(2, r.getNrOfCommands());
	}

	@Test
	public void findHighestAndLowest() {
		ServerCommandReplay r = new ServerCommandReplay();
		r.add(replayCommand(3));
		r.add(replayCommand(7));
		r.add(replayCommand(1));
		assertEquals(7, r.findHighestCommandNr());
		assertEquals(1, r.findLowestCommandNr());
	}

	@Test
	public void notReplayable() {
		assertFalse(new ServerCommandReplay().isReplayable());
	}

	@Test
	public void idIsServerReplay() {
		assertEquals(NetCommandId.SERVER_REPLAY, new ServerCommandReplay().getId());
	}

	@Test
	public void maxNrOfCommandsConstant() {
		assertEquals(100, ServerCommandReplay.MAX_NR_OF_COMMANDS);
	}

	@Test
	public void getIdIsServerReplay() {
		assertEquals(NetCommandId.SERVER_REPLAY, new ServerCommandReplay().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndCommandArray() {
		ServerCommandReplay r = new ServerCommandReplay();
		r.setCommandNr(4);
		r.setTotalNrOfCommands(50);
		r.setLastCommand(true);
		r.add(replayCommand(1));
		r.addMarkingAffectingCommand(2);
		JsonObject json = r.toJsonValue();
		assertEquals("serverReplay", json.get("netCommandId").asString());
		assertEquals(4, json.get("commandNr").asInt());
		assertEquals(50, json.get("totalNrOfCommands").asInt());
		assertTrue(json.get("lastCommand").asBoolean());
		assertEquals(1, json.get("commandArray").asArray().get(0).asObject().get("commandNr").asInt());
		assertEquals(2, json.get("markingIntervalIndexes").asArray().get(0).asInt());
	}

	@Test
	public void roundTripWithData() {
		ServerCommandReplay r = new ServerCommandReplay();
		r.setCommandNr(3);
		r.setTotalNrOfCommands(10);
		r.setLastCommand(true);
		r.add(replayCommand(5));
		r.add(replayCommand(6));
		r.addMarkingAffectingCommand(1);
		JsonObject json = r.toJsonValue();
		ServerCommandReplay restored = new ServerCommandReplay().initFrom(NetCommandTestUtil.gameSource(), json);
		// Discrepancy vs Rust: Java's ServerCommandReplay.initFrom does not restore the
		// command's own commandNr (it stays 0), while the Rust translation restores it.
		assertEquals(0, restored.getCommandNr());
		assertEquals(10, restored.getTotalNrOfCommands());
		assertTrue(restored.isLastCommand());
		assertEquals(2, restored.getNrOfCommands());
		assertEquals(5, restored.getReplayCommands()[0].getCommandNr());
		assertTrue(restored.getMarkingAffectingCommands().contains(1));
	}

	@Test
	public void roundTripWithDefaultEmpty() {
		ServerCommandReplay r = new ServerCommandReplay();
		JsonObject json = r.toJsonValue();
		ServerCommandReplay restored = new ServerCommandReplay().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals(0, restored.getNrOfCommands());
		assertFalse(restored.isLastCommand());
		assertTrue(restored.getMarkingAffectingCommands().isEmpty());
	}
}
