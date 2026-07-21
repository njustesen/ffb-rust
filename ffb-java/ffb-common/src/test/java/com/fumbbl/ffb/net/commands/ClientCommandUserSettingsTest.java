package com.fumbbl.ffb.net.commands;

import com.fumbbl.ffb.CommonProperty;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;

import com.eclipsesource.json.JsonObject;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_user_settings.rs tests.
 * Java stores a {@code Map<CommonProperty, String>}; JSON uses parallel sorted
 * {@code settingNames}/{@code settingValues} arrays.
 */
public class ClientCommandUserSettingsTest {

	@Test
	public void defaultEmpty() {
		assertEquals(0, new ClientCommandUserSettings().getSettingNames().length);
	}

	@Test
	public void setAndGetValue() {
		ClientCommandUserSettings cmd = new ClientCommandUserSettings();
		cmd.addSetting(CommonProperty.SETTING_SOUND_VOLUME, "80");
		assertEquals("80", cmd.getSettingValue(CommonProperty.SETTING_SOUND_VOLUME));
	}

	@Test
	public void getIdIsClientUserSettings() {
		assertEquals(NetCommandId.CLIENT_USER_SETTINGS, new ClientCommandUserSettings().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndSettingNames() {
		ClientCommandUserSettings cmd = new ClientCommandUserSettings();
		cmd.addSetting(CommonProperty.SETTING_SOUND_VOLUME, "80");
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientUserSettings", json.get("netCommandId").asString());
		assertEquals(CommonProperty.SETTING_SOUND_VOLUME.getKey(),
			json.get("settingNames").asArray().get(0).asString());
		assertEquals("80", json.get("settingValues").asArray().get(0).asString());
	}

	@Test
	public void roundTripWithSettingsAndEntropy() {
		ClientCommandUserSettings cmd = new ClientCommandUserSettings();
		cmd.setEntropy((byte) 4);
		cmd.addSetting(CommonProperty.SETTING_SOUND_VOLUME, "80");
		JsonObject json = cmd.toJsonValue();
		ClientCommandUserSettings restored = new ClientCommandUserSettings()
			.initFrom(NetCommandTestUtil.gameSource(), json);
		assertTrue(restored.hasEntropy());
		assertEquals((byte) 4, restored.getEntropy());
		assertEquals("80", restored.getSettingValue(CommonProperty.SETTING_SOUND_VOLUME));
	}

	@Test
	public void roundTripWithNoSettings() {
		ClientCommandUserSettings cmd = new ClientCommandUserSettings();
		JsonObject json = cmd.toJsonValue();
		ClientCommandUserSettings restored = new ClientCommandUserSettings()
			.initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals(0, restored.getSettingNames().length);
	}
}
