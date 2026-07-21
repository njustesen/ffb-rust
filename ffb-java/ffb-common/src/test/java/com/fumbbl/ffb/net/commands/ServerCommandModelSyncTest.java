package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.SoundId;
import com.fumbbl.ffb.model.change.ModelChangeList;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import com.fumbbl.ffb.report.ReportList;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/server_command_model_sync.rs tests.
 * The Rust-only test `size_of_is_at_least_zero` (std::mem::size_of) has no Java equivalent and is skipped.
 */
public class ServerCommandModelSyncTest {

	@Test
	public void fieldsStored() {
		ServerCommandModelSync cmd = new ServerCommandModelSync(new ModelChangeList(), new ReportList(), null,
			SoundId.TOUCHDOWN, 5000, 2000);
		assertEquals(SoundId.TOUCHDOWN, cmd.getSound());
		assertEquals(5000L, cmd.getGameTime());
		assertEquals(2000L, cmd.getTurnTime());
	}

	@Test
	public void defaultZeroTimes() {
		ServerCommandModelSync cmd = new ServerCommandModelSync();
		assertEquals(0L, cmd.getGameTime());
		assertEquals(0L, cmd.getTurnTime());
	}

	@Test
	public void getIdIsServerModelSync() {
		assertEquals(NetCommandId.SERVER_MODEL_SYNC, new ServerCommandModelSync().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndTimes() {
		ServerCommandModelSync cmd = new ServerCommandModelSync(new ModelChangeList(), new ReportList(), null,
			SoundId.TOUCHDOWN, 100, 50);
		cmd.setCommandNr(4);
		JsonObject json = cmd.toJsonValue();
		assertEquals("serverModelSync", json.get("netCommandId").asString());
		assertEquals(4, json.get("commandNr").asInt());
		assertEquals(100L, json.get("gameTime").asLong());
		assertEquals(50L, json.get("turnTime").asLong());
		assertEquals("touchdown", json.get("sound").asString());
	}

	@Test
	public void roundTripWithTimesAndSound() {
		ServerCommandModelSync cmd = new ServerCommandModelSync(new ModelChangeList(), new ReportList(), null,
			SoundId.BLOCK, 7000, 3000);
		cmd.setCommandNr(9);
		JsonObject json = cmd.toJsonValue();
		ServerCommandModelSync restored = new ServerCommandModelSync().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals(9, restored.getCommandNr());
		assertEquals(SoundId.BLOCK, restored.getSound());
		assertEquals(7000L, restored.getGameTime());
		assertEquals(3000L, restored.getTurnTime());
	}

	@Test
	public void roundTripWithDefaults() {
		ServerCommandModelSync cmd = new ServerCommandModelSync();
		JsonObject json = cmd.toJsonValue();
		ServerCommandModelSync restored = new ServerCommandModelSync().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals(0L, restored.getGameTime());
		assertEquals(0L, restored.getTurnTime());
		assertEquals(0, restored.getReportList().size());
	}
}
