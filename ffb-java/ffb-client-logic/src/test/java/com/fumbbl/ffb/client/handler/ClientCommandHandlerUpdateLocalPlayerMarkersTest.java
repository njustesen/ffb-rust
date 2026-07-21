package com.fumbbl.ffb.client.handler;

import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.marking.PlayerMarker;
import com.fumbbl.ffb.model.FieldModel;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.net.commands.ServerCommandUpdateLocalPlayerMarkers;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Answers;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.ArrayList;
import java.util.Collections;
import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;
import static org.mockito.Mockito.verify;

// Mirrors ffb-rust ffb-client crates/ffb-client/src/client/handler/client_command_handler_update_local_player_markers.rs
// (Rust: mod tests), one test fn per Rust #[test].
@ExtendWith(MockitoExtension.class)
class ClientCommandHandlerUpdateLocalPlayerMarkersTest {

	@Mock(answer = Answers.RETURNS_DEEP_STUBS)
	private FantasyFootballClient client;

	@Mock
	private Game game;

	private FieldModel fieldModel;
	private ClientCommandHandlerUpdateLocalPlayerMarkers handler;

	@BeforeEach
	void setUp() {
		fieldModel = new FieldModel(game);
		given(game.getFieldModel()).willReturn(fieldModel);
		given(client.getGame()).willReturn(game);
		handler = new ClientCommandHandlerUpdateLocalPlayerMarkers(client);
	}

	@Test
	void replacesExistingMarkersWithTheNewSet() {
		fieldModel.add(new PlayerMarker("old"));
		ServerCommandUpdateLocalPlayerMarkers command =
			new ServerCommandUpdateLocalPlayerMarkers(Collections.singletonList(new PlayerMarker("new")));

		assertTrue(handler.handleNetCommand(command, ClientCommandHandlerMode.PLAYING));

		PlayerMarker[] markers = fieldModel.getPlayerMarkers();
		assertEquals(1, markers.length);
		assertEquals("new", markers[0].getPlayerId());
		verify(client.getUserInterface()).refresh();
	}

	@Test
	void emptyMarkerListClearsAllMarkers() {
		fieldModel.add(new PlayerMarker("old"));
		ServerCommandUpdateLocalPlayerMarkers command =
			new ServerCommandUpdateLocalPlayerMarkers(new ArrayList<>());

		assertTrue(handler.handleNetCommand(command, ClientCommandHandlerMode.PLAYING));

		assertEquals(0, fieldModel.getPlayerMarkers().length);
	}

	// Rust: wrong_command_type_is_ignored_but_returns_true -- SKIPPED.
	// Java casts unconditionally; wrong type throws CCE, not a no-op.

	@Test
	void multipleNewMarkersAreAllAdded() {
		List<PlayerMarker> newMarkers = new ArrayList<>();
		newMarkers.add(new PlayerMarker("a"));
		newMarkers.add(new PlayerMarker("b"));
		ServerCommandUpdateLocalPlayerMarkers command = new ServerCommandUpdateLocalPlayerMarkers(newMarkers);

		assertTrue(handler.handleNetCommand(command, ClientCommandHandlerMode.PLAYING));

		assertEquals(2, fieldModel.getPlayerMarkers().length);
	}
}
