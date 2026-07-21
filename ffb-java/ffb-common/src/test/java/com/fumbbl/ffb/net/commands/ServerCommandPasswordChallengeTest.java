package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.FactoryType.FactoryContext;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/server_command_password_challenge.rs tests.
 */
public class ServerCommandPasswordChallengeTest {

	@Test
	public void challengeStored() {
		ServerCommandPasswordChallenge cmd = new ServerCommandPasswordChallenge("abc123");
		assertEquals("abc123", cmd.getChallenge());
	}

	@Test
	public void defaultEmpty() {
		ServerCommandPasswordChallenge cmd = new ServerCommandPasswordChallenge();
		assertNull(cmd.getChallenge());
	}

	@Test
	public void getIdIsServerPasswordChallenge() {
		assertEquals(NetCommandId.SERVER_PASSWORD_CHALLENGE, new ServerCommandPasswordChallenge().getId());
	}

	@Test
	public void getContextIsApplication() {
		assertEquals(FactoryContext.APPLICATION, new ServerCommandPasswordChallenge().getContext());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndChallenge() {
		ServerCommandPasswordChallenge cmd = new ServerCommandPasswordChallenge("abc123");
		JsonObject json = cmd.toJsonValue();
		assertEquals("serverPasswordChallenge", json.get("netCommandId").asString());
		assertEquals("abc123", json.get("challenge").asString());
	}

	@Test
	public void roundTripWithChallenge() {
		ServerCommandPasswordChallenge cmd = new ServerCommandPasswordChallenge("xyz789");
		cmd.setCommandNr(2);
		JsonObject json = cmd.toJsonValue();
		ServerCommandPasswordChallenge restored = new ServerCommandPasswordChallenge().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals(2, restored.getCommandNr());
		assertEquals("xyz789", restored.getChallenge());
	}

	@Test
	public void roundTripWithNoChallenge() {
		ServerCommandPasswordChallenge cmd = new ServerCommandPasswordChallenge();
		JsonObject json = cmd.toJsonValue();
		ServerCommandPasswordChallenge restored = new ServerCommandPasswordChallenge().initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getChallenge());
	}
}
