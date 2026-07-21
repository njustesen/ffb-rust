package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.CommonProperty;
import com.fumbbl.ffb.FactoryType.FactoryContext;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/server_command_user_settings.rs tests.
 */
public class ServerCommandUserSettingsTest {

	@Test
	public void addAndRetrieve() {
		ServerCommandUserSettings cmd = new ServerCommandUserSettings();
		cmd.addUserSetting(CommonProperty.SETTING_SOUND_MODE, "on");
		assertEquals("on", cmd.getUserSettingValue(CommonProperty.SETTING_SOUND_MODE));
	}

	@Test
	public void missingKeyReturnsNone() {
		ServerCommandUserSettings cmd = new ServerCommandUserSettings();
		assertNull(cmd.getUserSettingValue(CommonProperty.SETTING_SOUND_VOLUME));
	}

	@Test
	public void getIdIsServerUserSettings() {
		assertEquals(NetCommandId.SERVER_USER_SETTINGS, new ServerCommandUserSettings().getId());
	}

	@Test
	public void getContextIsApplication() {
		assertEquals(FactoryContext.APPLICATION, new ServerCommandUserSettings().getContext());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndNames() {
		ServerCommandUserSettings cmd = new ServerCommandUserSettings();
		cmd.addUserSetting(CommonProperty.SETTING_SOUND_MODE, "on");
		JsonObject json = cmd.toJsonValue();
		assertEquals("serverUserSettings", json.get("netCommandId").asString());
		assertEquals("setting.sound.mode", json.get("userSettingNames").asArray().get(0).asString());
		assertEquals("on", json.get("userSettingValues").asArray().get(0).asString());
	}

	@Test
	public void roundTripWithSettings() {
		ServerCommandUserSettings cmd = new ServerCommandUserSettings();
		cmd.setCommandNr(5);
		cmd.addUserSetting(CommonProperty.SETTING_SOUND_MODE, "on");
		cmd.addUserSetting(CommonProperty.SETTING_SOUND_VOLUME, "50");
		JsonObject json = cmd.toJsonValue();
		ServerCommandUserSettings restored = new ServerCommandUserSettings().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals(5, restored.getCommandNr());
		assertEquals("on", restored.getUserSettingValue(CommonProperty.SETTING_SOUND_MODE));
		assertEquals("50", restored.getUserSettingValue(CommonProperty.SETTING_SOUND_VOLUME));
	}

	@Test
	public void roundTripWithNoSettings() {
		ServerCommandUserSettings cmd = new ServerCommandUserSettings();
		JsonObject json = cmd.toJsonValue();
		ServerCommandUserSettings restored = new ServerCommandUserSettings().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals(0, restored.getUserSettingNames().length);
	}
}
