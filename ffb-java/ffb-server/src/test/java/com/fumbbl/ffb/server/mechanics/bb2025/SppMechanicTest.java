package com.fumbbl.ffb.server.mechanics.bb2025;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.mechanics.bb2025.SppMechanic;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.PlayerResult;
import com.fumbbl.ffb.model.SpecialRule;
import com.fumbbl.ffb.model.Team;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import java.util.Collections;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/mechanics/bb2025/spp_mechanic.rs tests. The BB2025 SPP
 * mechanic: a touchdown is worth 3 SPP (2 for a Brawlin' Brutes team) and a casualty 2 (3 for
 * Brawlin' Brutes); addLanding increments landings; addCatch increments the additional-SPP catch
 * counter when the player's team is in the additional-catch set. (Java addCatch reads the team from
 * pr.getPlayer().getTeam(); the Rust threads team_id explicitly — same result.)
 */
public class SppMechanicTest {

	private Game game;
	private final SppMechanic mechanic = new SppMechanic();

	@BeforeEach
	void setUp() {
		GameState gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2025);
		game = gameState.getGame();
	}

	// rust: touchdown_spp_normal_team_is_3
	@Test
	public void touchdownSppNormalTeamIs3() {
		assertEquals(3, mechanic.touchdownSpp(game.getTeamHome()));
	}

	// rust: touchdown_spp_brutes_team_is_2
	@Test
	public void touchdownSppBrutesTeamIs2() {
		Team brutes = game.getTeamAway();
		brutes.getSpecialRules().add(SpecialRule.BRAWLIN_BRUTES);
		assertEquals(2, mechanic.touchdownSpp(brutes));
	}

	// rust: casualty_spp_brutes_team_is_3
	@Test
	public void casualtySppBrutesTeamIs3() {
		Team brutes = game.getTeamAway();
		brutes.getSpecialRules().add(SpecialRule.BRAWLIN_BRUTES);
		assertEquals(3, mechanic.casualtySpp(brutes));
	}

	// rust: add_landing_increments_landings
	@Test
	public void addLandingIncrementsLandings() {
		PlayerResult pr = game.getGameResult().getPlayerResult(game.getPlayerById("home1"));
		mechanic.addLanding(pr);
		assertEquals(1, pr.getLandings());
	}

	// rust: add_catch_with_extra_team_increments_extra_catches
	@Test
	public void addCatchWithExtraTeamIncrementsExtraCatches() {
		PlayerResult pr = game.getGameResult().getPlayerResult(game.getPlayerById("home1"));
		mechanic.addCatch(Collections.singleton(game.getTeamHome().getId()), pr);
		assertEquals(1, pr.getCatchesWithAdditionalSpp());
	}
}
