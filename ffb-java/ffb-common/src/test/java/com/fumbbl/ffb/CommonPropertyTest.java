package com.fumbbl.ffb;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.EnumSource;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

class CommonPropertyTest {

	@ParameterizedTest
	@EnumSource(CommonProperty.class)
	public void assertFieldLength(CommonProperty property) {
		assertTrue(property.getKey().length() <= 40, "Name of " + property.name() + " is too long for database (40 chars)");
	}

	@Test
	public void getKeyReturnsDotSeparatedString() {
		assertEquals("client.command.compression", CommonProperty.CLIENT_COMMAND_COMPRESSION.getKey());
		assertEquals("setting.sound.mode", CommonProperty.SETTING_SOUND_MODE.getKey());
	}

	@Test
	public void forKeyRoundTrips() {
		assertEquals(CommonProperty.CLIENT_COMMAND_COMPRESSION, CommonProperty.forKey("client.command.compression"));
		assertNull(CommonProperty.forKey("invalid.key"));
	}

	@Test
	public void allIsNonEmpty() {
		assertTrue(CommonProperty.values().length > 0);
	}

	@Test
	public void isStoredRemoteDefaults() {
		assertTrue(CommonProperty.SETTING_LOCAL_SETTINGS.isStoredRemote());
		assertTrue(CommonProperty.CLIENT_COMMAND_COMPRESSION.isStoredRemote());
		assertFalse(CommonProperty.SETTING_SOUND_MODE.isStoredRemote());
	}

	@Test
	public void getValueReturnsNoneForClientPropsAndSomeForSettings() {
		assertNull(CommonProperty.CLIENT_PING_INTERVAL.getValue());
		assertNull(CommonProperty.HTTPCLIENT_TIMEOUT_CONNECT.getValue());
		assertEquals("Sound", CommonProperty.SETTING_SOUND_MODE.getValue());
		assertEquals("Autocomplete", CommonProperty.SETTING_AUTOCOMPLETE.getValue());
	}
}
