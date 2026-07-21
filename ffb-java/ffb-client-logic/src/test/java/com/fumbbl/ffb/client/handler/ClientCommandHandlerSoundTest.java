package com.fumbbl.ffb.client.handler;

import com.fumbbl.ffb.CommonProperty;
import com.fumbbl.ffb.IClientPropertyValue;
import com.fumbbl.ffb.SoundId;
import com.fumbbl.ffb.client.ClientReplayer;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.client.UserInterface;
import com.fumbbl.ffb.client.sound.SoundEngine;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.commands.ServerCommandSound;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Answers;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.mockito.quality.Strictness;
import org.mockito.junit.jupiter.MockitoSettings;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;

/**
 * Mirrors Rust {@code client_command_handler_sound.rs} tests. Rust's {@code last_sound} field
 * (invented purely to make the pure part of the handler testable) has no Java equivalent —
 * the Java handler plays the sound directly via {@code playSound}, so the port asserts on the
 * {@link SoundEngine#playSound(SoundId)} seam instead.
 */
@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class ClientCommandHandlerSoundTest {

	@Mock(answer = Answers.RETURNS_DEEP_STUBS)
	private FantasyFootballClient client;

	private ClientCommandHandlerSound handler;

	@Test
	void getIdReturnsServerSound() {
		handler = new ClientCommandHandlerSound(client);
		assertEquals(NetCommandId.SERVER_SOUND, handler.getId());
	}

	@Test
	void handleNetCommandPlaysTheSoundWhenPlayingAndSoundIsOn() {
		handler = new ClientCommandHandlerSound(client);
		given(client.getProperty(CommonProperty.SETTING_SOUND_MODE)).willReturn(IClientPropertyValue.SETTING_SOUND_ON);
		SoundEngine soundEngine = client.getUserInterface().getSoundEngine();

		ServerCommandSound cmd = new ServerCommandSound(SoundId.BLOCK);

		assertTrue(handler.handleNetCommand(cmd, ClientCommandHandlerMode.PLAYING));

		verify(soundEngine).playSound(SoundId.BLOCK);
	}

	@Test
	void handleNetCommandDoesNotPlayWhenSoundSettingIsOff() {
		handler = new ClientCommandHandlerSound(client);
		given(client.getProperty(CommonProperty.SETTING_SOUND_MODE)).willReturn(IClientPropertyValue.SETTING_SOUND_OFF);
		SoundEngine soundEngine = client.getUserInterface().getSoundEngine();

		ServerCommandSound cmd = new ServerCommandSound(SoundId.CATCH);

		assertTrue(handler.handleNetCommand(cmd, ClientCommandHandlerMode.PLAYING));

		verify(soundEngine, never()).playSound(SoundId.CATCH);
	}

	@Test
	void handleNetCommandReturnsTrueAcrossAllModesWithoutNpe() {
		handler = new ClientCommandHandlerSound(client);
		ClientReplayer replayer = client.getReplayer();
		given(replayer.isReplayingSingleSpeedForward()).willReturn(true);
		given(client.getProperty(CommonProperty.SETTING_SOUND_MODE)).willReturn(IClientPropertyValue.SETTING_SOUND_ON);

		ServerCommandSound cmd = new ServerCommandSound(SoundId.CATCH);

		assertTrue(handler.handleNetCommand(cmd, ClientCommandHandlerMode.REPLAYING));
		assertTrue(handler.handleNetCommand(cmd, ClientCommandHandlerMode.QUEUING));
	}

	// Rust `handle_net_command_ignores_mismatched_command_type` SKIPPED: Java casts
	// `(ServerCommandSound) pNetCommand` unconditionally; a wrong command type throws
	// ClassCastException instead of no-op'ing.
}
