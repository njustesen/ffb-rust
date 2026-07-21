package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_password_challenge.rs tests.
 */
public class ClientCommandPasswordChallengeTest {

	@Test
	public void defaultHasNoCoach() {
		ClientCommandPasswordChallenge cmd = new ClientCommandPasswordChallenge();
		assertNull(cmd.getCoach());
	}

	@Test
	public void withCoachStoresValue() {
		ClientCommandPasswordChallenge cmd = new ClientCommandPasswordChallenge("coach-xyz");
		assertEquals("coach-xyz", cmd.getCoach());
	}

	@Test
	public void getIdIsClientPasswordChallenge() {
		assertEquals(NetCommandId.CLIENT_PASSWORD_CHALLENGE, new ClientCommandPasswordChallenge().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndCoach() {
		ClientCommandPasswordChallenge cmd = new ClientCommandPasswordChallenge("coach1");
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientPasswordChallenge", json.get("netCommandId").asString());
		assertEquals("coach1", json.get("coach").asString());
	}

	@Test
	public void roundTripWithData() {
		ClientCommandPasswordChallenge cmd = new ClientCommandPasswordChallenge("coach2");
		cmd.setEntropy((byte) 11);
		JsonObject json = cmd.toJsonValue();
		ClientCommandPasswordChallenge restored =
			new ClientCommandPasswordChallenge().initFrom(NetCommandTestUtil.gameSource(), json);
		assertTrue(restored.hasEntropy());
		assertEquals((byte) 11, restored.getEntropy());
		assertEquals("coach2", restored.getCoach());
	}

	@Test
	public void roundTripDefault() {
		ClientCommandPasswordChallenge cmd = new ClientCommandPasswordChallenge();
		JsonObject json = cmd.toJsonValue();
		ClientCommandPasswordChallenge restored =
			new ClientCommandPasswordChallenge().initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getCoach());
		assertFalse(restored.hasEntropy());
	}
}
