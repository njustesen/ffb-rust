package com.fumbbl.ffb.server.mechanics.bb2020;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.mechanics.bb2020.SppMechanic;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.PlayerResult;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import java.util.Collections;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/mechanics/bb2020/spp_mechanic.rs tests. BB2020: MVP 4,
 * deflection 1; addCompletion/addCasualty increment the base counter and, when the player's team is
 * in the additional-SPP set, the additional counter. (Java's add* methods read the team via
 * pr.getPlayer().getTeam(); the Rust threads team_id — same result.)
 */
public class SppMechanicTest {

	private Game game;
	private final SppMechanic mechanic = new SppMechanic();

	@BeforeEach
	void setUp() {
		GameState gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
		game = gameState.getGame();
	}

	private PlayerResult homePlayerResult() {
		return game.getGameResult().getPlayerResult(game.getPlayerById("home1"));
	}

	// rust: mvp_spp_is_4
	@Test
	public void mvpSppIs4() {
		assertEquals(4, mechanic.mvpSpp());
	}

	// rust: add_completion_with_extra_team_sets_extra_flag
	@Test
	public void addCompletionWithExtraTeamSetsExtraFlag() {
		PlayerResult pr = homePlayerResult();
		mechanic.addCompletion(Collections.singleton(game.getTeamHome().getId()), pr);
		assertEquals(1, pr.getCompletions());
		assertEquals(1, pr.getCompletionsWithAdditionalSpp());
	}

	// rust: add_completion_without_extra_team_no_extra_flag
	@Test
	public void addCompletionWithoutExtraTeamNoExtraFlag() {
		PlayerResult pr = homePlayerResult();
		mechanic.addCompletion(Collections.emptySet(), pr);
		assertEquals(1, pr.getCompletions());
		assertEquals(0, pr.getCompletionsWithAdditionalSpp());
	}

	// rust: add_casualty_with_extra_team_sets_extra_flag
	@Test
	public void addCasualtyWithExtraTeamSetsExtraFlag() {
		PlayerResult pr = homePlayerResult();
		mechanic.addCasualty(Collections.singleton(game.getTeamHome().getId()), pr);
		assertEquals(1, pr.getCasualties());
		assertEquals(1, pr.getCasualtiesWithAdditionalSpp());
	}

	// rust: deflection_spp_is_1
	@Test
	public void deflectionSppIs1() {
		assertEquals(1, mechanic.deflectionSpp());
	}
}
