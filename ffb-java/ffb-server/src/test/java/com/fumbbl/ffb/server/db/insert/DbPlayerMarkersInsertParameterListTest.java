package com.fumbbl.ffb.server.db.insert;

import com.fumbbl.ffb.marking.PlayerMarker;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import org.junit.jupiter.api.Test;

import java.util.Arrays;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust {@code db_player_markers_insert_parameter_list} tests.
 *
 * <p>The Rust {@code init_from_missing_team_ids}, {@code init_from_not_started}
 * and {@code trait_get_parameters_boxes_clones} cases are Rust-structural (empty
 * team ids / no-driver GameState / trait boxing) and are not ported: the
 * {@link GameFixture} always builds started teams with ids, and Java has no trait
 * boxing.
 */
class DbPlayerMarkersInsertParameterListTest {

	@Test
	void initFromNullGameStateIsNoop() {
		DbPlayerMarkersInsertParameterList list = new DbPlayerMarkersInsertParameterList();
		list.initFrom(null, false, false);
		assertEquals(0, list.getParameters().length);
	}

	@Test
	void addParameter() {
		DbPlayerMarkersInsertParameterList list = new DbPlayerMarkersInsertParameterList();
		list.addParameter(new DbPlayerMarkersInsertParameter("t1", "p1", "text"));
		assertEquals(1, list.getParameters().length);
	}

	@Test
	void initFromAddsHomeAndAwayParameters() {
		GameState gameState = GameFixture.createGameState();
		PlayerMarker markerHome = new PlayerMarker("home1");
		markerHome.setHomeText("Home note");
		PlayerMarker markerAway = new PlayerMarker("away1");
		markerAway.setAwayText("Away note");
		gameState.getGame().getFieldModel().add(markerHome);
		gameState.getGame().getFieldModel().add(markerAway);

		DbPlayerMarkersInsertParameterList list = new DbPlayerMarkersInsertParameterList();
		list.initFrom(gameState, false, false);

		DbPlayerMarkersInsertParameter[] params = list.getParameters();
		assertEquals(2, params.length);
		assertTrue(Arrays.stream(params).anyMatch(p -> "home".equals(p.getTeamId())
			&& "home1".equals(p.getPlayerId()) && "Home note".equals(p.getText())));
		assertTrue(Arrays.stream(params).anyMatch(p -> "away".equals(p.getTeamId())
			&& "away1".equals(p.getPlayerId()) && "Away note".equals(p.getText())));
	}

	@Test
	void initFromSkipsEmptyTextAndAutoFlags() {
		GameState gameState = GameFixture.createGameState();
		PlayerMarker markerHome = new PlayerMarker("home1");
		markerHome.setHomeText("Home note");
		gameState.getGame().getFieldModel().add(markerHome);

		DbPlayerMarkersInsertParameterList list = new DbPlayerMarkersInsertParameterList();
		// loadAutoHome = true means home markers are skipped.
		list.initFrom(gameState, true, false);
		assertEquals(0, list.getParameters().length);
	}
}
