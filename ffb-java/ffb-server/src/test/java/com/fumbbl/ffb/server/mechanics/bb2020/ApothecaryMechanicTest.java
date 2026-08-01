package com.fumbbl.ffb.server.mechanics.bb2020;

import com.fumbbl.ffb.ApothecaryType;
import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.PlayerType;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.mechanics.bb2020.ApothecaryMechanic;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.RosterPlayer;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/mechanics/bb2020/apothecary_mechanic.rs tests. BB2020
 * apothecaryTypes: STAR (or zapped) → empty; a regular player gets TEAM when the team has more
 * apothecaries than wandering ones, else WANDERING if any; PLAGUE only when the team has a plague
 * doctor AND the player is KO'd; a mercenary can only use a wandering apothecary.
 */
public class ApothecaryMechanicTest {

	private Game game;
	private RosterPlayer defender;
	private final ApothecaryMechanic mechanic = new ApothecaryMechanic();

	@BeforeEach
	void setUp() {
		GameState gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
		game = gameState.getGame();
		defender = (RosterPlayer) game.getPlayerById("home1");
		defender.setType(PlayerType.REGULAR);
	}

	private List<ApothecaryType> types(PlayerState state) {
		return mechanic.apothecaryTypes(game, defender, state);
	}

	// rust: star_player_returns_empty
	@Test
	public void starPlayerReturnsEmpty() {
		defender.setType(PlayerType.STAR);
		assertTrue(types(new PlayerState(PlayerState.SERIOUS_INJURY)).isEmpty());
	}

	// rust: regular_player_with_team_apo_returns_team_type
	@Test
	public void regularPlayerWithTeamApoReturnsTeamType() {
		game.getTurnDataHome().setApothecaries(1);
		assertEquals(List.of(ApothecaryType.TEAM), types(new PlayerState(PlayerState.SERIOUS_INJURY)));
	}

	// rust: regular_player_ko_with_plague_doctor_gets_plague_type
	@Test
	public void regularPlayerKoWithPlagueDoctorGetsPlagueType() {
		game.getTurnDataHome().setPlagueDoctors(1);
		assertTrue(types(new PlayerState(PlayerState.KNOCKED_OUT)).contains(ApothecaryType.PLAGUE));
	}

	// rust: no_apo_returns_empty
	@Test
	public void noApoReturnsEmpty() {
		assertTrue(types(new PlayerState(PlayerState.SERIOUS_INJURY)).isEmpty());
	}

	// rust: mercenary_player_with_wandering_apo_returns_wandering
	@Test
	public void mercenaryPlayerWithWanderingApoReturnsWandering() {
		defender.setType(PlayerType.MERCENARY);
		game.getTurnDataHome().setWanderingApothecaries(1);
		assertTrue(types(new PlayerState(PlayerState.SERIOUS_INJURY)).contains(ApothecaryType.WANDERING));
	}

	// rust: plague_doctor_not_applied_to_serious_injury_for_regular
	@Test
	public void plagueDoctorNotAppliedToSeriousInjuryForRegular() {
		game.getTurnDataHome().setPlagueDoctors(1);
		assertFalse(types(new PlayerState(PlayerState.SERIOUS_INJURY)).contains(ApothecaryType.PLAGUE));
	}
}
