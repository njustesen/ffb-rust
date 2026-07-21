package com.fumbbl.ffb.client.state.logic;

import com.fumbbl.ffb.client.ClientParameters;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.client.net.ClientCommunication;
import com.fumbbl.ffb.model.ActingPlayer;
import com.fumbbl.ffb.net.commands.ServerCommandVersion;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Answers;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.mockito.junit.jupiter.MockitoSettings;
import org.mockito.quality.Strictness;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

// Mirrors ffb-rust ffb-client crates/ffb-client/src/client/state/logic/login_logic_module.rs
// (Rust: mod tests), one test fn per Rust #[test] that is faithfully reproducible.
//
// SKIPPED (with reasons):
// - create_response_is_always_none: Java's `createResponse(String)` is `private`, not
//   accessible from a same-package test. Additionally its real behavior is a genuine
//   MD5-based `PasswordChallenge.createResponse` call (not a stub returning null like the
//   Rust port's documented gap), so it wouldn't be "always None" anyway even if callable.
@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class LoginLogicModuleTest {

	@Mock(answer = Answers.RETURNS_DEEP_STUBS)
	FantasyFootballClient client;

	@Test
	void checkVersionConflictTrueWhenActualMajorLower() {
		LoginLogicModule module = new LoginLogicModule(client);

		assertTrue(module.checkVersionConflict("2.0.0", "1.9.9"));
	}

	@Test
	void checkVersionConflictFalseWhenEqual() {
		LoginLogicModule module = new LoginLogicModule(client);

		assertFalse(module.checkVersionConflict("1.2.3", "1.2.3"));
	}

	@Test
	void checkVersionConflictFalseOnUnparseableVersion() {
		LoginLogicModule module = new LoginLogicModule(client);

		// java: unmatched regex leaves both major/minor/release at 0, so no conflict.
		assertFalse(module.checkVersionConflict("not-a-version", "also-not"));
	}

	@Test
	void initCommunicationUsesTeamIdWhenProvided() {
		ClientParameters parameters = client.getParameters();
		when(parameters.getTeamId()).thenReturn("42");
		when(parameters.getTeamName()).thenReturn("Orcs");
		LoginLogicModule module = new LoginLogicModule(client);

		module.initCommunication();

		assertEquals("Orcs", module.getTeamHomeName());
		assertNull(module.getTeamAwayName());
		verify(client.getCommunication()).sendRequestVersion();
	}

	@Test
	void initCommunicationUsesTeamHomeAndAwayWithoutTeamId() {
		ClientParameters parameters = client.getParameters();
		when(parameters.getTeamId()).thenReturn(null);
		when(parameters.getTeamHome()).thenReturn("Orcs");
		when(parameters.getTeamAway()).thenReturn("Elves");
		LoginLogicModule module = new LoginLogicModule(client);

		module.initCommunication();

		assertEquals("Orcs", module.getTeamHomeName());
		assertEquals("Elves", module.getTeamAwayName());
	}

	@Test
	void idAndNameProvidedRequiresZeroGameIdAndGameName() {
		ClientParameters parameters = client.getParameters();
		when(parameters.getGameId()).thenReturn(0L);
		when(parameters.getAuthentication()).thenReturn(null);
		LoginLogicModule module = new LoginLogicModule(client);

		assertFalse(module.idAndNameProvided());

		module.sendChallenge(new LoginLogicModule.LoginData("LocalGame", null, 0, false));

		assertTrue(module.idAndNameProvided());
	}

	@Test
	void handleVersionCommandSuccessWhenVersionsMatch() {
		LoginLogicModule module = new LoginLogicModule(client);
		String version = com.fumbbl.ffb.FantasyFootballConstants.VERSION;
		ServerCommandVersion cmd = new ServerCommandVersion(version, version, null, null, false);

		assertEquals(LoginLogicModule.VersionCheck.SUCCESS, module.handleVersionCommand(cmd));
	}

	@Test
	void handleVersionCommandClientFailWhenClientTooOld() {
		LoginLogicModule module = new LoginLogicModule(client);
		String version = com.fumbbl.ffb.FantasyFootballConstants.VERSION;
		ServerCommandVersion cmd = new ServerCommandVersion(version, "99.0.0", null, null, false);

		assertEquals(LoginLogicModule.VersionCheck.CLIENT_FAIL, module.handleVersionCommand(cmd));
	}

	@Test
	void actionContextPanics() {
		LoginLogicModule module = new LoginLogicModule(client);
		ActingPlayer actingPlayer = new ActingPlayer(null);

		assertThrows(UnsupportedOperationException.class, () -> module.actionContext(actingPlayer));
	}
}
