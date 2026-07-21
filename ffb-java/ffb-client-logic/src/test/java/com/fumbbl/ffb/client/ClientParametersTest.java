package com.fumbbl.ffb.client;

import com.fumbbl.ffb.ClientMode;
import com.fumbbl.ffb.FantasyFootballException;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertThrows;

class ClientParametersTest {

	@Test
	void noModeFailsValidation() {
		assertNull(ClientParameters.createValidParams(new String[]{"-coach", "bob"}));
	}

	@Test
	void playerWithoutCoachFails() {
		assertNull(ClientParameters.createValidParams(new String[]{"-player"}));
	}

	@Test
	void playerWithCoachSucceeds() {
		ClientParameters params = ClientParameters.createValidParams(new String[]{"-player", "-coach", "bob"});
		assertNotNull(params);
		assertEquals(ClientMode.PLAYER, params.getMode());
		assertEquals("bob", params.getCoach());
	}

	@Test
	void playerWithGameIdAndOnlyTeamHomeFails() {
		ClientParameters result = ClientParameters.createValidParams(new String[]{
			"-player", "-coach", "bob", "-gameId", "1", "-teamHome", "home",
		});
		assertNull(result);
	}

	@Test
	void playerWithGameIdAndBothTeamsSucceeds() {
		ClientParameters result = ClientParameters.createValidParams(new String[]{
			"-player", "-coach", "bob", "-gameId", "1", "-teamHome", "home", "-teamAway", "away",
		});
		assertNotNull(result);
	}

	@Test
	void playerWithTeamIdButNoTeamNameFails() {
		ClientParameters result = ClientParameters.createValidParams(new String[]{
			"-player", "-coach", "bob", "-teamId", "1",
		});
		assertNull(result);
	}

	@Test
	void spectatorRequiresCoach() {
		assertNull(ClientParameters.createValidParams(new String[]{"-spectator"}));
		assertNotNull(ClientParameters.createValidParams(new String[]{"-spectator", "-coach", "bob"}));
	}

	@Test
	void replayRequiresPositiveGameId() {
		assertNull(ClientParameters.createValidParams(new String[]{"-replay"}));
		ClientParameters ok = ClientParameters.createValidParams(new String[]{"-replay", "-gameId", "42"});
		assertNotNull(ok);
		assertEquals(42, ok.getGameId());
	}

	// DISCREPANCY: Rust's parse() returns a Result and create_valid_params() maps Err to None,
	// so the Rust tests of the same name assert `is_none()`. Java's private constructor throws
	// FantasyFootballException (a RuntimeException) directly and createValidParams() does not
	// catch it, so the exception propagates to the caller instead of yielding null.
	@Test
	void nonNumericGameIdThrows() {
		assertThrows(FantasyFootballException.class,
			() -> ClientParameters.createValidParams(new String[]{"-replay", "-gameId", "abc"}));
	}

	@Test
	void unknownArgumentThrows() {
		assertThrows(FantasyFootballException.class,
			() -> ClientParameters.createValidParams(new String[]{"-bogus"}));
	}

	@Test
	void trailingArgumentMissingValueThrows() {
		assertThrows(FantasyFootballException.class,
			() -> ClientParameters.createValidParams(new String[]{"-replay", "-gameId"}));
	}

	@Test
	void layoutDefaultsToLandscape() {
		ClientParameters params = ClientParameters.createValidParams(new String[]{"-spectator", "-coach", "bob"});
		assertNotNull(params);
		assertEquals(ClientLayout.LANDSCAPE, params.getLayout());
	}

	@Test
	void layoutArgumentIsParsed() {
		ClientParameters params = ClientParameters.createValidParams(new String[]{
			"-spectator", "-coach", "bob", "-layout", "PORTRAIT",
		});
		assertNotNull(params);
		assertEquals(ClientLayout.PORTRAIT, params.getLayout());
	}

	@Test
	void portAndServerAndBuildRoundTrip() {
		ClientParameters params = ClientParameters.createValidParams(new String[]{
			"-spectator", "-coach", "bob", "-port", "1234", "-server", "example.com", "-build", "42",
		});
		assertNotNull(params);
		assertEquals(1234, params.getPort());
		assertEquals("example.com", params.getServer());
		assertEquals("42", params.getBuild());
	}
}
