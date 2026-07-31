package com.fumbbl.ffb.server.mechanic.bb2025;

import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.LeaderState;
import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.RosterPlayer;
import com.fumbbl.ffb.model.TurnData;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.InjuryResult;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.StepId;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/mechanic/bb2025/state_mechanic.rs tests
 * (core startHalf / updateLeaderReRolls / handlePumpUp subset — the inducement-reset and
 * once-per-half skill-reset tests follow in a separate pass).
 */
public class StateMechanicTest {

	private GameState gameState;
	private Game game;
	private IStep step;
	private StateMechanic mechanic;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3);
		game = gameState.getGame();
		step = GameFixture.createStep(gameState, StepId.INIT_START_GAME);
		mechanic = new StateMechanic();
	}

	// rust: start_half_sets_half_counter
	@Test
	public void startHalfSetsHalfCounter() {
		mechanic.startHalf(step, 2);
		assertEquals(2, game.getHalf());
	}

	// rust: start_half_resets_turn_numbers
	@Test
	public void startHalfResetsTurnNumbers() {
		game.getTurnDataHome().setTurnNr(4);
		game.getTurnDataAway().setTurnNr(3);
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

	// rust: start_half_home_playing_home_first_offense
	@Test
	public void startHalfHomePlayingHomeFirstOffense() {
		game.setHomeFirstOffense(true);
		mechanic.startHalf(step, 1);
		assertFalse(game.isHomePlaying());
		mechanic.startHalf(step, 2);
		assertTrue(game.isHomePlaying());
	}

	// rust: start_half_home_playing_away_first_offense
	@Test
	public void startHalfHomePlayingAwayFirstOffense() {
		game.setHomeFirstOffense(false);
		mechanic.startHalf(step, 1);
		assertTrue(game.isHomePlaying());
		mechanic.startHalf(step, 2);
		assertFalse(game.isHomePlaying());
	}

	// rust: start_half_resets_leader_state_at_or_before_half_2
	@Test
	public void startHalfResetsLeaderStateAtOrBeforeHalf2() {
		game.getTurnDataHome().setLeaderState(LeaderState.AVAILABLE);
		game.getTurnDataAway().setLeaderState(LeaderState.USED);
		mechanic.startHalf(step, 2);
		assertEquals(LeaderState.NONE, game.getTurnDataHome().getLeaderState());
		assertEquals(LeaderState.NONE, game.getTurnDataAway().getLeaderState());
	}

	// rust: start_half_does_not_reset_leader_at_half_3
	@Test
	public void startHalfDoesNotResetLeaderAtHalf3() {
		game.getTurnDataHome().setLeaderState(LeaderState.AVAILABLE);
		mechanic.startHalf(step, 3);
		assertEquals(LeaderState.AVAILABLE, game.getTurnDataHome().getLeaderState());
	}

	// rust: update_leader_none_no_leader_on_field_no_change
	@Test
	public void updateLeaderNoneNoLeaderOnFieldNoChange() {
		TurnData turnData = game.getTurnDataHome();
		turnData.setReRolls(2);
		mechanic.updateLeaderReRollsForTeam(turnData, game.getTeamHome(), game.getFieldModel(), step);
		assertEquals(LeaderState.NONE, turnData.getLeaderState());
		assertEquals(2, turnData.getReRolls());
	}

	// rust: update_leader_available_to_none_no_leader_on_field
	@Test
	public void updateLeaderAvailableToNoneNoLeaderOnField() {
		TurnData turnData = game.getTurnDataHome();
		turnData.setLeaderState(LeaderState.AVAILABLE);
		turnData.setReRolls(3);
		mechanic.updateLeaderReRollsForTeam(turnData, game.getTeamHome(), game.getFieldModel(), step);
		assertEquals(LeaderState.NONE, turnData.getLeaderState());
		assertEquals(2, turnData.getReRolls());
	}

	// rust: update_leader_used_state_no_change
	@Test
	public void updateLeaderUsedStateNoChange() {
		TurnData turnData = game.getTurnDataHome();
		turnData.setLeaderState(LeaderState.USED);
		turnData.setReRolls(1);
		mechanic.updateLeaderReRollsForTeam(turnData, game.getTeamHome(), game.getFieldModel(), step);
		assertEquals(LeaderState.USED, turnData.getLeaderState());
		assertEquals(1, turnData.getReRolls());
	}

	// rust: update_leader_rerolls_not_negative
	@Test
	public void updateLeaderRerollsNotNegative() {
		TurnData turnData = game.getTurnDataHome();
		turnData.setLeaderState(LeaderState.AVAILABLE);
		turnData.setReRolls(0);
		mechanic.updateLeaderReRollsForTeam(turnData, game.getTeamHome(), game.getFieldModel(), step);
		assertEquals(0, turnData.getReRolls());
	}

	private InjuryResult blockInjuryResult(boolean casualty, com.fumbbl.ffb.injury.InjuryType injuryType) {
		InjuryResult injuryResult = new InjuryResult();
		injuryResult.injuryContext().setAttackerId("home1");
		injuryResult.injuryContext().setInjury(
			new PlayerState(casualty ? PlayerState.RIP : PlayerState.STUNNED));
		injuryResult.injuryContext().setInjuryType(injuryType);
		return injuryResult;
	}

	private void setUpPumpUpAttacker() {
		game.setHomePlaying(true);
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		((RosterPlayer) game.getPlayerById("home1"))
			.addSkill(GameFixture.skill(game, "Pump Up The Crowd"));
	}

	// rust: handle_pump_up_no_attacker_returns_false
	@Test
	public void handlePumpUpNoAttackerReturnsFalse() {
		assertFalse(mechanic.handlePumpUp(step, new InjuryResult()));
	}

	// rust: handle_pump_up_non_block_injury_type_returns_false
	@Test
	public void handlePumpUpNonBlockInjuryTypeReturnsFalse() {
		setUpPumpUpAttacker();
		assertFalse(mechanic.handlePumpUp(step, blockInjuryResult(true, new com.fumbbl.ffb.injury.Foul())));
	}

	// rust: handle_pump_up_no_injury_type_returns_false
	@Test
	public void handlePumpUpNoInjuryTypeReturnsFalse() {
		setUpPumpUpAttacker();
		assertFalse(mechanic.handlePumpUp(step, blockInjuryResult(true, null)));
	}

	// rust: handle_pump_up_block_casualty_grants_reroll
	@Test
	public void handlePumpUpBlockCasualtyGrantsReroll() {
		setUpPumpUpAttacker();
		boolean result = mechanic.handlePumpUp(step, blockInjuryResult(true, new com.fumbbl.ffb.injury.Block()));
		assertTrue(result);
		assertEquals(1, game.getTurnDataHome().getReRolls());
		assertEquals(1, game.getTurnDataHome().getReRollsPumpUpTheCrowdOneDrive());
	}

	// rust: start_half_re_rolls_set_from_team_first_two_halves
	@Test
	public void startHalfReRollsSetFromTeamFirstTwoHalves() {
		game.getTeamHome().setReRolls(3);
		game.getTeamAway().setReRolls(2);
		mechanic.startHalf(step, 1);
		assertEquals(3, game.getTurnDataHome().getReRolls());
		assertEquals(2, game.getTurnDataAway().getReRolls());
	}

	// rust: start_half_apothecaries_only_at_half_1
	@Test
	public void startHalfApothecariesOnlyAtHalf1() {
		game.getTeamHome().setApothecaries(1);
		game.getTurnDataHome().setApothecaries(0);
		mechanic.startHalf(step, 2);
		assertEquals(0, game.getTurnDataHome().getApothecaries());
	}

	// rust: start_half_apothecaries_set_at_half_1
	@Test
	public void startHalfApothecariesSetAtHalf1() {
		game.getTeamHome().setApothecaries(2);
		mechanic.startHalf(step, 1);
		assertEquals(2, game.getTurnDataHome().getApothecaries());
	}

	// ── skill / inducement resets (Java: private helpers exercised through startHalf) ────────

	private com.fumbbl.ffb.model.skill.Skill addAndMarkUsed(String playerId, String skillName) {
		RosterPlayer player = (RosterPlayer) game.getPlayerById(playerId);
		com.fumbbl.ffb.model.skill.Skill skill = GameFixture.skill(game, skillName);
		player.addSkill(skill);
		player.markUsed(skill, game);
		return skill;
	}

	// rust: reset_special_skills_clears_once_per_half_skill_for_all_players
	@Test
	public void resetSpecialSkillsClearsOncePerHalfSkillForAllPlayers() {
		com.fumbbl.ffb.model.skill.Skill leader = addAndMarkUsed("home1", "Leader");
		com.fumbbl.ffb.model.skill.Skill beerBarrelBash = addAndMarkUsed("away1", "Beer Barrel Bash!");
		mechanic.startHalf(step, 1);
		assertFalse(game.getPlayerById("home1").isUsed(leader), "ONCE_PER_HALF Leader reset");
		assertFalse(game.getPlayerById("away1").isUsed(beerBarrelBash), "ONCE_PER_DRIVE reset too");
	}

	// rust: reset_special_skills_at_half_3_skips_once_per_half_but_clears_end_of_drive
	@Test
	public void resetSpecialSkillsAtHalf3SkipsOncePerHalfButClearsEndOfDrive() {
		com.fumbbl.ffb.model.skill.Skill leader = addAndMarkUsed("home1", "Leader");
		com.fumbbl.ffb.model.skill.Skill beerBarrelBash = addAndMarkUsed("home1", "Beer Barrel Bash!");
		mechanic.startHalf(step, 3);
		assertTrue(game.getPlayerById("home1").isUsed(leader), "half > 2: ONCE_PER_HALF not reset");
		assertFalse(game.getPlayerById("home1").isUsed(beerBarrelBash), "ONCE_PER_DRIVE always reset");
	}

	private com.fumbbl.ffb.inducement.Inducement addInducement(TurnData turnData, String typeName,
															   int value, int uses) {
		com.fumbbl.ffb.inducement.InducementType type =
			(com.fumbbl.ffb.inducement.InducementType) game
				.getFactory(com.fumbbl.ffb.FactoryType.Factory.INDUCEMENT_TYPE).forName(typeName);
		com.fumbbl.ffb.inducement.Inducement inducement =
			new com.fumbbl.ffb.inducement.Inducement(type, value);
		inducement.setUses(uses);
		turnData.getInducementSet().addInducement(inducement);
		return inducement;
	}

	private int conditionalRerollUses(TurnData turnData) {
		com.fumbbl.ffb.inducement.InducementType type =
			turnData.getInducementSet().forUsage(com.fumbbl.ffb.inducement.Usage.CONDITIONAL_REROLL);
		return turnData.getInducementSet().get(type).getUses();
	}

	// rust: reset_inducements_resets_conditional_reroll_uses
	@Test
	public void resetInducementsResetsConditionalRerollUses() {
		addInducement(game.getTurnDataHome(), "teamMascot", 2, 2);
		mechanic.startHalf(step, 1);
		assertEquals(0, conditionalRerollUses(game.getTurnDataHome()));
	}

	// rust: reset_inducements_no_op_when_no_conditional_reroll
	@Test
	public void resetInducementsNoOpWhenNoConditionalReroll() {
		mechanic.startHalf(step, 1); // no conditional-reroll inducement — must not throw
		assertEquals(1, game.getHalf());
	}

	// rust: reset_inducements_skips_at_half_3
	@Test
	public void resetInducementsSkipsAtHalf3() {
		addInducement(game.getTurnDataHome(), "teamMascot", 2, 2);
		mechanic.startHalf(step, 3);
		assertEquals(2, conditionalRerollUses(game.getTurnDataHome()), "half > 2: uses not reset");
	}

	// rust: reset_inducements_resets_away_team_too
	@Test
	public void resetInducementsResetsAwayTeamToo() {
		addInducement(game.getTurnDataAway(), "teamMascot", 1, 1);
		mechanic.startHalf(step, 2);
		assertEquals(0, conditionalRerollUses(game.getTurnDataAway()));
	}

	// rust: start_half_returns_inducement_events_for_wandering_apo_at_half_1
	@Test
	public void startHalfReturnsInducementEventsForWanderingApoAtHalf1() {
		addInducement(game.getTurnDataHome(), "wanderingApothecaries", 1, 0);
		mechanic.startHalf(step, 1);
		assertTrue(step.getResult().getReportList().hasReport(com.fumbbl.ffb.report.ReportId.INDUCEMENT),
			"wandering apothecary should produce an inducement report");
	}

	// rust: start_half_no_inducements_returns_empty_events
	@Test
	public void startHalfNoInducementsReturnsEmptyEvents() {
		mechanic.startHalf(step, 1);
		assertFalse(step.getResult().getReportList().hasReport(com.fumbbl.ffb.report.ReportId.INDUCEMENT),
			"no inducements should produce no inducement report");
	}

	// rust: start_half_half_2_skips_apothecary_events
	@Test
	public void startHalfHalf2SkipsApothecaryEvents() {
		addInducement(game.getTurnDataHome(), "wanderingApothecaries", 1, 0);
		mechanic.startHalf(step, 2);
		assertFalse(step.getResult().getReportList().hasReport(com.fumbbl.ffb.report.ReportId.INDUCEMENT),
			"apothecaries not registered at half 2");
	}
}
