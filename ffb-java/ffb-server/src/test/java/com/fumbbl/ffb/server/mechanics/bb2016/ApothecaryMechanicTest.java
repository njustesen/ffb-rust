package com.fumbbl.ffb.server.mechanics.bb2016;

import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.mechanics.Mechanic;
import com.fumbbl.ffb.mechanics.bb2016.ApothecaryMechanic;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/mechanics/bb2016/apothecary_mechanic.rs tests. In BB2016
 * apothecaryTypes always returns an empty list (apothecary use is not modelled per-type here); the
 * mechanic reports type/name APOTHECARY. (The Rust default_creates_instance is Rust-structural
 * plumbing; apothecary_types_empty_for_star_player_too is redundant in BB2016 — the result is
 * unconditionally empty regardless of player type/state — so the star/type distinction is exercised
 * in the bb2020/bb2025 twins instead.)
 */
public class ApothecaryMechanicTest {

	private Game game;
	private final ApothecaryMechanic mechanic = new ApothecaryMechanic();

	@BeforeEach
	void setUp() {
		GameState gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
		game = gameState.getGame();
	}

	// rust: apothecary_types_always_empty_in_bb2016
	@Test
	public void apothecaryTypesAlwaysEmpty() {
		assertTrue(mechanic.apothecaryTypes(game, game.getPlayerById("home1"),
			new PlayerState(PlayerState.SERIOUS_INJURY)).isEmpty());
	}

	// rust: apothecary_types_empty_regardless_of_player_state
	@Test
	public void apothecaryTypesEmptyRegardlessOfPlayerState() {
		assertTrue(mechanic.apothecaryTypes(game, game.getPlayerById("home1"),
			new PlayerState(PlayerState.KNOCKED_OUT)).isEmpty());
	}

	// rust: mechanic_type_is_apothecary
	@Test
	public void mechanicTypeIsApothecary() {
		assertEquals(Mechanic.Type.APOTHECARY, mechanic.getType());
	}

	// rust: get_name_returns_apothecary_string
	@Test
	public void getNameReturnsApothecaryString() {
		assertEquals("APOTHECARY", mechanic.getName());
	}
}
