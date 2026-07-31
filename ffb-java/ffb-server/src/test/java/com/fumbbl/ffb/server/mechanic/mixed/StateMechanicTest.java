package com.fumbbl.ffb.server.mechanic.mixed;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.LeaderState;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.inducement.Inducement;
import com.fumbbl.ffb.inducement.InducementType;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.TurnData;
import com.fumbbl.ffb.report.ReportId;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.InjuryResult;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.StepId;
import com.fumbbl.ffb.server.util.UtilServerGame;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import java.util.Arrays;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/mechanic/mixed/state_mechanic.rs tests (BB2016/BB2020).
 * The Rust handle_chef_rolls lives on the mechanic; Java's equivalent is
 * UtilServerGame.handleChefRolls (invoked from this mechanic's startHalf) — tested here against
 * that utility with scripted dice (3d6 per chef, each face &gt; 3 steals one re-roll).
 */
public class StateMechanicTest {

	private GameState gameState;
	private Game game;
	private IStep step;
	private StateMechanic mechanic;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
		game = gameState.getGame();
		step = GameFixture.createStep(gameState, StepId.INIT_START_GAME);
		mechanic = new StateMechanic();
	}

	private void addMasterChef(TurnData turnData, int value) {
		InducementType type = (InducementType) game
			.getFactory(FactoryType.Factory.INDUCEMENT_TYPE).forName("halflingMasterChef");
		turnData.getInducementSet().addInducement(new Inducement(type, value));
	}

	private long masterChefReportCount() {
		return Arrays.stream(step.getResult().getReportList().getReports())
			.filter(r -> r.getId() == ReportId.MASTER_CHEF_ROLL).count();
	}

	// rust: start_half_sets_half_counter
	@Test
	public void startHalfSetsHalfCounter() {
		mechanic.startHalf(step, 1);
		assertEquals(1, game.getHalf());
	}

	// rust: start_half_resets_turn_numbers
	@Test
	public void startHalfResetsTurnNumbers() {
		game.getTurnDataHome().setTurnNr(7);
		game.getTurnDataAway().setTurnNr(6);
		mechanic.startHalf(step, 1);
		assertEquals(0, game.getTurnDataHome().getTurnNr());
		assertEquals(0, game.getTurnDataAway().getTurnNr());
	}

	// rust: start_half_clears_ball
	@Test
	public void startHalfClearsBall() {
		game.getFieldModel().setBallCoordinate(new FieldCoordinate(5, 5));
		game.getFieldModel().setBallInPlay(true);
		mechanic.startHalf(step, 1);
		assertFalse(game.getFieldModel().isBallInPlay());
		assertNull(game.getFieldModel().getBallCoordinate());
	}

	// rust: start_half_home_playing_logic
	@Test
	public void startHalfHomePlayingLogic() {
		game.setHomeFirstOffense(true);
		mechanic.startHalf(step, 1);
		assertFalse(game.isHomePlaying());
		mechanic.startHalf(step, 2);
		assertTrue(game.isHomePlaying());
	}

	// rust: start_half_resets_leader_state_at_halftime
	@Test
	public void startHalfResetsLeaderStateAtHalftime() {
		game.getTurnDataHome().setLeaderState(LeaderState.AVAILABLE);
		game.getTurnDataAway().setLeaderState(LeaderState.USED);
		mechanic.startHalf(step, 2);
		assertEquals(LeaderState.NONE, game.getTurnDataHome().getLeaderState());
		assertEquals(LeaderState.NONE, game.getTurnDataAway().getLeaderState());
	}

	// rust: start_half_sets_re_rolls_first_two_halves
	@Test
	public void startHalfSetsReRollsFirstTwoHalves() {
		game.getTeamHome().setReRolls(4);
		mechanic.startHalf(step, 1);
		assertEquals(4, game.getTurnDataHome().getReRolls());
	}

	// rust: start_half_no_re_rolls_half_3
	@Test
	public void startHalfNoReRollsHalf3() {
		game.getTeamHome().setReRolls(4);
		game.getTurnDataHome().setReRolls(0);
		mechanic.startHalf(step, 3);
		assertEquals(0, game.getTurnDataHome().getReRolls());
	}

	// rust: start_half_apothecaries_only_half_0
	@Test
	public void startHalfApothecariesOnlyHalf0() {
		game.getTeamHome().setApothecaries(2);
		mechanic.startHalf(step, 1);
		assertEquals(2, game.getTurnDataHome().getApothecaries());
		game.getTurnDataHome().setApothecaries(0);
		mechanic.startHalf(step, 2);
		assertEquals(0, game.getTurnDataHome().getApothecaries());
	}

	// rust: update_leader_available_to_none_no_leader_on_field
	@Test
	public void updateLeaderAvailableToNoneNoLeaderOnField() {
		TurnData turnData = game.getTurnDataHome();
		turnData.setLeaderState(LeaderState.AVAILABLE);
		turnData.setReRolls(2);
		mechanic.updateLeaderReRollsForTeam(turnData, game.getTeamHome(), game.getFieldModel(), step);
		assertEquals(LeaderState.NONE, turnData.getLeaderState());
		assertEquals(1, turnData.getReRolls());
	}

	// rust: update_leader_no_change_when_used
	@Test
	public void updateLeaderNoChangeWhenUsed() {
		TurnData turnData = game.getTurnDataHome();
		turnData.setLeaderState(LeaderState.USED);
		mechanic.updateLeaderReRollsForTeam(turnData, game.getTeamHome(), game.getFieldModel(), step);
		assertEquals(LeaderState.USED, turnData.getLeaderState());
	}

	// rust: handle_pump_up_no_attacker_returns_false
	@Test
	public void handlePumpUpNoAttackerReturnsFalse() {
		assertFalse(mechanic.handlePumpUp(step, new InjuryResult()));
	}

	// rust: handle_chef_rolls_no_chefs_no_change
	@Test
	public void handleChefRollsNoChefsNoChange() {
		game.getTurnDataHome().setReRolls(3);
		game.getTurnDataAway().setReRolls(2);
		UtilServerGame.handleChefRolls(step, game);
		assertEquals(0, masterChefReportCount(), "no chefs -> no chef reports");
		assertEquals(3, game.getTurnDataHome().getReRolls());
		assertEquals(2, game.getTurnDataAway().getReRolls());
	}

	// rust: handle_chef_rolls_emits_event_per_roll
	@Test
	public void handleChefRollsEmitsEventPerRoll() {
		addMasterChef(game.getTurnDataHome(), 2);
		GameFixture.installScriptedDice(gameState, 1, 1, 1, 1, 1, 1); // 2 chefs x 3d6, no steals
		UtilServerGame.handleChefRolls(step, game);
		assertEquals(2, masterChefReportCount(), "2 chefs -> 2 chef reports");
	}

	// rust: handle_chef_rolls_adjusts_rerolls_symmetrically
	@Test
	public void handleChefRollsAdjustsRerollsSymmetrically() {
		addMasterChef(game.getTurnDataAway(), 1);
		game.getTurnDataHome().setReRolls(3);
		game.getTurnDataAway().setReRolls(2);
		GameFixture.installScriptedDice(gameState, 6, 5, 1); // faces > 3 steal -> 2 stolen
		UtilServerGame.handleChefRolls(step, game);
		assertEquals(1, game.getTurnDataHome().getReRolls(), "home loses the stolen re-rolls");
		assertEquals(4, game.getTurnDataAway().getReRolls(), "away gains the stolen re-rolls");
	}

	// rust: handle_chef_rolls_home_rerolls_floored_at_zero
	@Test
	public void handleChefRollsHomeRerollsFlooredAtZero() {
		addMasterChef(game.getTurnDataAway(), 1);
		game.getTurnDataHome().setReRolls(0);
		game.getTurnDataAway().setReRolls(0);
		GameFixture.installScriptedDice(gameState, 6, 6, 6); // 3 stolen
		UtilServerGame.handleChefRolls(step, game);
		assertEquals(0, game.getTurnDataHome().getReRolls(), "home floored at zero");
		assertEquals(3, game.getTurnDataAway().getReRolls());
	}
}
