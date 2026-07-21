package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_select_weather.rs tests.
 */
public class ClientCommandSelectWeatherTest {

	@Test
	public void defaultHasZeroModifierAndNoName() {
		ClientCommandSelectWeather cmd = new ClientCommandSelectWeather();
		assertEquals(0, cmd.getModifier());
		assertNull(cmd.getWeatherName());
	}

	@Test
	public void withFieldsStoresValues() {
		ClientCommandSelectWeather cmd = new ClientCommandSelectWeather(2, "Nice");
		assertEquals(2, cmd.getModifier());
		assertEquals("Nice", cmd.getWeatherName());
	}

	@Test
	public void negativeModifierStored() {
		ClientCommandSelectWeather cmd = new ClientCommandSelectWeather(-1, "Sweltering Heat");
		assertEquals(-1, cmd.getModifier());
		assertEquals("Sweltering Heat", cmd.getWeatherName());
	}

	@Test
	public void getIdIsClientSelectWeather() {
		assertEquals(NetCommandId.CLIENT_SELECT_WEATHER, new ClientCommandSelectWeather().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndModifier() {
		ClientCommandSelectWeather cmd = new ClientCommandSelectWeather(3, "Sunny");
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientSelectWeather", json.get("netCommandId").asString());
		assertEquals(3, json.get("modifier").asInt());
		assertEquals("Sunny", json.get("name").asString());
	}

	@Test
	public void roundTripPopulated() {
		ClientCommandSelectWeather cmd = new ClientCommandSelectWeather(-1, "Sweltering Heat");
		cmd.setEntropy((byte) 11);
		JsonObject json = cmd.toJsonValue();
		ClientCommandSelectWeather restored = new ClientCommandSelectWeather().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals(-1, restored.getModifier());
		assertEquals("Sweltering Heat", restored.getWeatherName());
		assertEquals((byte) 11, restored.getEntropy());
	}

	@Test
	public void roundTripDefault() {
		ClientCommandSelectWeather cmd = new ClientCommandSelectWeather();
		JsonObject json = cmd.toJsonValue();
		ClientCommandSelectWeather restored = new ClientCommandSelectWeather().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals(0, restored.getModifier());
		assertNull(restored.getWeatherName());
		assertFalse(restored.hasEntropy());
	}
}
