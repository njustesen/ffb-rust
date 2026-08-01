package com.fumbbl.ffb.server.mechanics.bb2016;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.mechanics.bb2016.SppMechanic;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.PlayerResult;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import java.util.Collections;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/mechanics/bb2016/spp_mechanic.rs tests. BB2016: MVP 5,
 * touchdown 3 (flat — no Brawlin' Brutes), deflection 1; addCompletion/addCasualty increment the
 * base counters.
 */
public class SppMechanicTest {

	private Game game;
	private final SppMechanic mechanic = new SppMechanic();

	@BeforeEach
	void setUp() {
		GameState gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
		game = gameState.getGame();
	}

	private PlayerResult homePlayerResult() {
		return game.getGameResult().getPlayerResult(game.getPlayerById("home1"));
	}

	// rust: mvp_spp_is_5
	@Test
	public void mvpSppIs5() {
		assertEquals(5, mechanic.mvpSpp());
	}

	// rust: touchdown_spp_is_3
	@Test
	public void touchdownSppIs3() {
		assertEquals(3, mechanic.touchdownSpp(game.getTeamHome()));
	}

	// rust: add_completion_increments_completions
	@Test
	public void addCompletionIncrementsCompletions() {
		PlayerResult pr = homePlayerResult();
		mechanic.addCompletion(Collections.emptySet(), pr);
		assertEquals(1, pr.getCompletions());
	}

	// rust: add_casualty_increments_casualties
	@Test
	public void addCasualtyIncrementsCasualties() {
		PlayerResult pr = homePlayerResult();
		mechanic.addCasualty(Collections.emptySet(), pr);
		assertEquals(1, pr.getCasualties());
	}

	// rust: deflection_spp_is_1
	@Test
	public void deflectionSppIs1() {
		assertEquals(1, mechanic.deflectionSpp());
	}
}
