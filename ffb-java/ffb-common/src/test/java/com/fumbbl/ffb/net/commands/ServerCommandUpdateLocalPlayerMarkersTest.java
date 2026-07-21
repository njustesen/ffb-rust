package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.marking.PlayerMarker;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import java.util.Collections;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/server_command_update_local_player_markers.rs tests.
 */
public class ServerCommandUpdateLocalPlayerMarkersTest {

	@Test
	public void fieldsStored() {
		ServerCommandUpdateLocalPlayerMarkers cmd = new ServerCommandUpdateLocalPlayerMarkers(Collections.singletonList(new PlayerMarker()));
		assertEquals(1, cmd.getMarkers().size());
	}

	@Test
	public void defaultEmpty() {
		ServerCommandUpdateLocalPlayerMarkers cmd = new ServerCommandUpdateLocalPlayerMarkers();
		assertTrue(cmd.getMarkers().isEmpty());
	}

	@Test
	public void getIdIsServerUpdateLocalPlayerMarkers() {
		assertEquals(NetCommandId.SERVER_UPDATE_LOCAL_PLAYER_MARKERS, new ServerCommandUpdateLocalPlayerMarkers().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndMarkers() {
		ServerCommandUpdateLocalPlayerMarkers cmd = new ServerCommandUpdateLocalPlayerMarkers(Collections.singletonList(new PlayerMarker("p1")));
		JsonObject json = cmd.toJsonValue().asObject();
		assertEquals("serverUpdateLocalPlayerMarkers", json.get("netCommandId").asString());
		assertEquals("p1", json.get("playerMarkerArray").asArray().get(0).asObject().get("playerId").asString());
	}

	@Test
	public void roundTripWithMarkers() {
		PlayerMarker marker = new PlayerMarker("p1");
		marker.setHomeText("H");
		marker.setAwayText("A");
		ServerCommandUpdateLocalPlayerMarkers cmd = new ServerCommandUpdateLocalPlayerMarkers(Collections.singletonList(marker));
		cmd.setCommandNr(1);
		JsonObject json = cmd.toJsonValue().asObject();
		ServerCommandUpdateLocalPlayerMarkers restored = new ServerCommandUpdateLocalPlayerMarkers().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals(1, restored.getCommandNr());
		assertEquals(1, restored.getMarkers().size());
		assertEquals("p1", restored.getMarkers().get(0).getPlayerId());
		assertEquals("H", restored.getMarkers().get(0).getHomeText());
		assertEquals("A", restored.getMarkers().get(0).getAwayText());
	}

	@Test
	public void roundTripWithNoMarkers() {
		ServerCommandUpdateLocalPlayerMarkers cmd = new ServerCommandUpdateLocalPlayerMarkers();
		JsonObject json = cmd.toJsonValue().asObject();
		ServerCommandUpdateLocalPlayerMarkers restored = new ServerCommandUpdateLocalPlayerMarkers().initFrom(NetCommandTestUtil.gameSource(), json);
		assertTrue(restored.getMarkers().isEmpty());
	}
}
