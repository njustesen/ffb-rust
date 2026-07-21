package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.SoundId;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/server_command_sound.rs tests.
 */
public class ServerCommandSoundTest {

	@Test
	public void soundStored() {
		ServerCommandSound cmd = new ServerCommandSound(SoundId.TOUCHDOWN);
		assertEquals(SoundId.TOUCHDOWN, cmd.getSound());
	}

	@Test
	public void getIdIsServerSound() {
		assertEquals(NetCommandId.SERVER_SOUND, new ServerCommandSound().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndSound() {
		ServerCommandSound cmd = new ServerCommandSound(SoundId.BLOCK);
		JsonObject json = cmd.toJsonValue();
		assertEquals("serverSound", json.get("netCommandId").asString());
		assertEquals("block", json.get("sound").asString());
	}

	@Test
	public void roundTripWithSound() {
		ServerCommandSound cmd = new ServerCommandSound(SoundId.CATCH);
		cmd.setCommandNr(3);
		JsonObject json = cmd.toJsonValue();
		ServerCommandSound restored = new ServerCommandSound().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals(3, restored.getCommandNr());
		assertEquals(SoundId.CATCH, restored.getSound());
	}

	@Test
	public void roundTripDefault() {
		ServerCommandSound cmd = new ServerCommandSound(SoundId.TOUCHDOWN);
		JsonObject json = cmd.toJsonValue();
		ServerCommandSound restored = new ServerCommandSound().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals(SoundId.TOUCHDOWN, restored.getSound());
		assertEquals(0, restored.getCommandNr());
	}
}
