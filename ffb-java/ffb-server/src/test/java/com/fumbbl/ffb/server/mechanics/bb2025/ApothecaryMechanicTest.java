package com.fumbbl.ffb.server.mechanics.bb2025;

import com.fumbbl.ffb.ApothecaryType;
import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.mechanics.Mechanic;
import com.fumbbl.ffb.mechanics.bb2025.ApothecaryMechanic;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/mechanics/bb2025/apothecary_mechanic.rs tests. BB2025
 * apothecaryTypes simplifies BB2020: no STAR exclusion and no player-type branching, plague doctor
 * is NOT gated on KO (playerState is ignored). TEAM when apothecaries > wandering, else WANDERING if
 * any; PLAGUE whenever the team has a plague doctor.
 */
public class ApothecaryMechanicTest {

	private Game game;
	private Player<?> defender;
	private final ApothecaryMechanic mechanic = new ApothecaryMechanic();

	@BeforeEach
	void setUp() {
		GameState gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2025);
		game = gameState.getGame();
		defender = game.getPlayerById("home1");
	}

	private List<ApothecaryType> types() {
		return mechanic.apothecaryTypes(game, defender, new PlayerState(PlayerState.SERIOUS_INJURY));
	}

	// rust: no_apo_returns_empty
	@Test
	public void noApoReturnsEmpty() {
		assertTrue(types().isEmpty());
	}

	// rust: team_apo_returns_team_type
	@Test
	public void teamApoReturnsTeamType() {
		game.getTurnDataHome().setApothecaries(1);
		assertEquals(List.of(ApothecaryType.TEAM), types());
	}

	// rust: plague_doctor_returns_plague_type (BB2025: not gated on KO — serious injury still gets PLAGUE)
	@Test
	public void plagueDoctorReturnsPlagueType() {
		game.getTurnDataHome().setPlagueDoctors(1);
		assertTrue(types().contains(ApothecaryType.PLAGUE));
	}

	// rust: wandering_apo_when_no_team_apo (apothecaries == wandering -> wandering branch)
	@Test
	public void wanderingApoWhenNoTeamApo() {
		game.getTurnDataHome().setApothecaries(1);
		game.getTurnDataHome().setWanderingApothecaries(1);
		assertTrue(types().contains(ApothecaryType.WANDERING));
	}

	// rust: both_team_apo_and_plague_doctor
	@Test
	public void bothTeamApoAndPlagueDoctor() {
		game.getTurnDataHome().setApothecaries(1);
		game.getTurnDataHome().setPlagueDoctors(1);
		List<ApothecaryType> result = types();
		assertTrue(result.contains(ApothecaryType.TEAM));
		assertTrue(result.contains(ApothecaryType.PLAGUE));
	}

	// rust: mechanic_type_is_apothecary
	@Test
	public void mechanicTypeIsApothecary() {
		assertEquals(Mechanic.Type.APOTHECARY, mechanic.getType());
	}
}
