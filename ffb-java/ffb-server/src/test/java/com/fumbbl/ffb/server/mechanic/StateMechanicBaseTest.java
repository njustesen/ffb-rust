package com.fumbbl.ffb.server.mechanic;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.inducement.Inducement;
import com.fumbbl.ffb.inducement.InducementType;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.RosterPlayer;
import com.fumbbl.ffb.model.TurnData;
import com.fumbbl.ffb.report.IReport;
import com.fumbbl.ffb.report.ReportId;
import com.fumbbl.ffb.report.ReportInducement;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.InjuryResult;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.StepId;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import java.util.Arrays;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/mechanic/state_mechanic.rs (trait/base) tests.
 * Lives in the base package to reach the protected addApothecaries/addReRolls; driven through
 * the concrete bb2025 subclass (the base methods themselves are edition-agnostic).
 */
public class StateMechanicBaseTest {

	private GameState gameState;
	private Game game;
	private IStep step;
	private com.fumbbl.ffb.server.mechanic.bb2025.StateMechanic mechanic;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3);
		game = gameState.getGame();
		step = GameFixture.createStep(gameState, StepId.INIT_START_GAME);
		mechanic = new com.fumbbl.ffb.server.mechanic.bb2025.StateMechanic();
	}

	private Inducement addInducement(TurnData turnData, String typeName, int value) {
		InducementType type = (InducementType) game
			.getFactory(FactoryType.Factory.INDUCEMENT_TYPE).forName(typeName);
		Inducement inducement = new Inducement(type, value);
		turnData.getInducementSet().addInducement(inducement);
		return inducement;
	}

	private long inducementReportCount() {
		return Arrays.stream(step.getResult().getReportList().getReports())
			.filter(r -> r instanceof ReportInducement).count();
	}

	private ReportInducement firstInducementReport() {
		for (IReport report : step.getResult().getReportList().getReports()) {
			if (report instanceof ReportInducement) {
				return (ReportInducement) report;
			}
		}
		return null;
	}

	// rust: report_injury_emits_to_report_list
	@Test
	public void reportInjuryEmitsToReportList() {
		InjuryResult injuryResult = new InjuryResult();
		injuryResult.injuryContext().setDefenderId("away1");
		injuryResult.injuryContext().setInjuryType(new com.fumbbl.ffb.injury.Block());
		mechanic.reportInjury(step, injuryResult);
		assertTrue(injuryResult.isAlreadyReported());
		assertTrue(step.getResult().getReportList().hasReport(ReportId.INJURY));
	}

	// rust: report_injury_skips_second_call_when_already_reported
	@Test
	public void reportInjurySkipsSecondCallWhenAlreadyReported() {
		InjuryResult injuryResult = new InjuryResult();
		injuryResult.injuryContext().setDefenderId("away1");
		injuryResult.injuryContext().setInjuryType(new com.fumbbl.ffb.injury.Block());
		mechanic.reportInjury(step, injuryResult);
		assertEquals(1, step.getResult().getReportList().size());
		mechanic.reportInjury(step, injuryResult);
		assertEquals(1, step.getResult().getReportList().size());
	}

	// rust: reset_special_skill_does_not_panic
	@Test
	public void resetSpecialSkillDoesNotPanic() {
		mechanic.resetSpecialSkillAtEndOfDrive(game);
	}

	// rust: add_apothecaries_sets_from_team
	@Test
	public void addApothecariesSetsFromTeam() {
		game.getTeamHome().setApothecaries(2);
		mechanic.addApothecaries(step, true);
		assertEquals(2, game.getTurnDataHome().getApothecaries());
	}

	// rust: add_apothecaries_away_team
	@Test
	public void addApothecariesAwayTeam() {
		game.getTeamAway().setApothecaries(1);
		mechanic.addApothecaries(step, false);
		assertEquals(1, game.getTurnDataAway().getApothecaries());
	}

	// rust: add_re_rolls_sets_from_team
	@Test
	public void addReRollsSetsFromTeam() {
		game.getTeamHome().setReRolls(3);
		mechanic.addReRolls(step, true);
		assertEquals(3, game.getTurnDataHome().getReRolls());
	}

	// rust: add_apothecaries_adds_wandering_from_inducement_set
	@Test
	public void addApothecariesAddsWanderingFromInducementSet() {
		game.getTeamHome().setApothecaries(1);
		addInducement(game.getTurnDataHome(), "wanderingApothecaries", 1);
		mechanic.addApothecaries(step, true);
		assertEquals(2, game.getTurnDataHome().getApothecaries());
		assertEquals(1, game.getTurnDataHome().getWanderingApothecaries());
	}

	// rust: add_apothecaries_adds_plague_doctors
	@Test
	public void addApothecariesAddsPlagueDoctors() {
		addInducement(game.getTurnDataHome(), "plagueDoctor", 2);
		mechanic.addApothecaries(step, true);
		assertEquals(2, game.getTurnDataHome().getPlagueDoctors());
	}

	// rust: add_re_rolls_adds_extra_training
	@Test
	public void addReRollsAddsExtraTraining() {
		game.getTeamHome().setReRolls(2);
		addInducement(game.getTurnDataHome(), "extraTeamTraining", 1);
		mechanic.addReRolls(step, true);
		assertEquals(3, game.getTurnDataHome().getReRolls());
	}

	// rust: add_re_rolls_away_with_extra_training
	@Test
	public void addReRollsAwayWithExtraTraining() {
		game.getTeamAway().setReRolls(1);
		addInducement(game.getTurnDataAway(), "extraTeamTraining", 2);
		mechanic.addReRolls(step, false);
		assertEquals(3, game.getTurnDataAway().getReRolls());
	}

	// rust: team_has_leader_on_field_no_players
	@Test
	public void teamHasLeaderOnFieldNoPlayers() {
		assertFalse(gameState.hasLeader(game.getTeamHome()), "all players in reserve");
	}

	// rust: team_has_leader_on_field_with_player_in_box
	@Test
	public void teamHasLeaderOnFieldWithPlayerInBox() {
		((RosterPlayer) game.getPlayerById("home1")).addSkill(GameFixture.skill(game, "Leader"));
		assertFalse(gameState.hasLeader(game.getTeamHome()), "leader in reserve box is not on field");
	}

	// rust: team_has_leader_on_field_no_leader_skill
	@Test
	public void teamHasLeaderOnFieldNoLeaderSkill() {
		GameFixture.placePlayer(gameState, "home1", 8, 5);
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.MOVE);
		assertFalse(gameState.hasLeader(game.getTeamHome()), "player on pitch without Leader skill");
	}

	// ── report emission (Rust GameEvent::Inducement ↔ Java ReportInducement) ─────────────────

	// rust: add_apothecaries_no_inducement_emits_no_events
	@Test
	public void addApothecariesNoInducementEmitsNoEvents() {
		mechanic.addApothecaries(step, true);
		assertEquals(0, inducementReportCount());
	}

	// rust: add_apothecaries_wandering_emits_inducement_event
	@Test
	public void addApothecariesWanderingEmitsInducementEvent() {
		addInducement(game.getTurnDataHome(), "wanderingApothecaries", 1);
		mechanic.addApothecaries(step, true);
		assertEquals(1, inducementReportCount());
		ReportInducement report = firstInducementReport();
		assertEquals(1, report.getValue());
	}

	// rust: add_apothecaries_plague_doctor_emits_inducement_event
	// DOCUMENTED DIVERGENCE: Java's plague-doctor branch only sets turnData.plagueDoctors and
	// adds NO ReportInducement; the Rust GameEvent::Inducement for plague doctors is part of the
	// Rust wire layer (client sync happens via TurnData in Java). Java-true expectation: no report.
	@Test
	public void addApothecariesPlagueDoctorEmitsNoReport() {
		addInducement(game.getTurnDataHome(), "plagueDoctor", 2);
		mechanic.addApothecaries(step, true);
		assertEquals(0, inducementReportCount());
		assertEquals(2, game.getTurnDataHome().getPlagueDoctors());
	}

	// rust: add_apothecaries_wandering_and_plague_emits_two_events
	// (same divergence — Java reports only the wandering apothecary)
	@Test
	public void addApothecariesWanderingAndPlagueEmitsOneReport() {
		addInducement(game.getTurnDataHome(), "wanderingApothecaries", 1);
		addInducement(game.getTurnDataHome(), "plagueDoctor", 1);
		mechanic.addApothecaries(step, true);
		assertEquals(1, inducementReportCount());
	}

	// rust: add_apothecaries_away_team_event_carries_away_team_id
	@Test
	public void addApothecariesAwayTeamEventCarriesAwayTeamId() {
		addInducement(game.getTurnDataAway(), "wanderingApothecaries", 1);
		mechanic.addApothecaries(step, false);
		assertEquals(1, inducementReportCount());
		assertEquals(game.getTeamAway().getId(), firstInducementReport().getTeamId());
	}

	// rust: add_re_rolls_no_inducement_emits_no_events
	@Test
	public void addReRollsNoInducementEmitsNoEvents() {
		mechanic.addReRolls(step, true);
		assertEquals(0, inducementReportCount());
	}

	// rust: add_re_rolls_extra_training_emits_inducement_event
	@Test
	public void addReRollsExtraTrainingEmitsInducementEvent() {
		addInducement(game.getTurnDataHome(), "extraTeamTraining", 1);
		mechanic.addReRolls(step, true);
		assertEquals(1, inducementReportCount());
		assertEquals(1, firstInducementReport().getValue());
	}

	// rust: add_re_rolls_away_team_event_carries_away_team_id
	@Test
	public void addReRollsAwayTeamEventCarriesAwayTeamId() {
		addInducement(game.getTurnDataAway(), "extraTeamTraining", 2);
		mechanic.addReRolls(step, false);
		assertEquals(1, inducementReportCount());
		assertEquals(game.getTeamAway().getId(), firstInducementReport().getTeamId());
	}
}
