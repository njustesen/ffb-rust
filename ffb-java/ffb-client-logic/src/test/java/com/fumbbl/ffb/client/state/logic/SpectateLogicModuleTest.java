package com.fumbbl.ffb.client.state.logic;

import com.fumbbl.ffb.ClientMode;
import com.fumbbl.ffb.ClientStateId;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.model.ActingPlayer;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.Player;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.mockito.junit.jupiter.MockitoSettings;
import org.mockito.quality.Strictness;

import java.util.Date;

import static org.junit.jupiter.api.Assertions.assertDoesNotThrow;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.BDDMockito.given;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;

@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class SpectateLogicModuleTest {

	@Mock
	private FantasyFootballClient client;

	@Mock
	private Game game;

	@Mock
	private ActingPlayer actingPlayer;

	@Mock
	private Player<?> player;

	@BeforeEach
	public void setUp() {
		given(client.getGame()).willReturn(game);
	}

	// NOTE (test equalization): testGetIdReturnsSpectate / testAvailableActionsIsEmpty /
	// testPerformAvailableActionIsNoOp pruned — trivial LogicModule-subclass boilerplate with no Rust
	// twin (Rust SpectateLogicModule tests only canSwitchToSpectate/setUp behavior + actionContext
	// panic). Rust-as-reference.

	@Test
	public void testActionContextThrows() {
		SpectateLogicModule module = new SpectateLogicModule(client);
		UnsupportedOperationException exception = assertThrows(UnsupportedOperationException.class,
			() -> module.actionContext(actingPlayer));
		assertEquals("actionContext for acting player is not supported in spectate context", exception.getMessage());
	}

	@Test
	public void testCanSwitchToSpectateRequiresFinishedGameAndPlayerMode() {
		SpectateLogicModule module = new SpectateLogicModule(client);
		given(game.getFinished()).willReturn(null);
		given(client.getMode()).willReturn(ClientMode.PLAYER);
		assertFalse(module.canSwitchToSpectate());

		given(game.getFinished()).willReturn(new Date());
		assertTrue(module.canSwitchToSpectate());
	}

	@Test
	public void testCanSwitchToSpectateFalseWhenNotPlayerMode() {
		SpectateLogicModule module = new SpectateLogicModule(client);
		given(game.getFinished()).willReturn(new Date());
		given(client.getMode()).willReturn(ClientMode.SPECTATOR);
		assertFalse(module.canSwitchToSpectate());
	}

	@Test
	public void testSetUpSwitchesModeWhenEligible() {
		SpectateLogicModule module = new SpectateLogicModule(client);
		given(game.getFinished()).willReturn(new Date());
		given(client.getMode()).willReturn(ClientMode.PLAYER);

		module.setUp();

		verify(client).setMode(ClientMode.SPECTATOR);
	}

	@Test
	public void testSetUpDoesNotSwitchModeWhenNotEligible() {
		SpectateLogicModule module = new SpectateLogicModule(client);
		given(game.getFinished()).willReturn(null);
		given(client.getMode()).willReturn(ClientMode.PLAYER);

		module.setUp();

		verify(client, never()).setMode(any());
	}
}
