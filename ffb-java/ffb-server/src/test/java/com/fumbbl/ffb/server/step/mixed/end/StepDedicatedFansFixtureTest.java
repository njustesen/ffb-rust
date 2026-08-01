package com.fumbbl.ffb.server.step.mixed.end;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.StepAction;
import com.fumbbl.ffb.server.step.StepId;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/step/mixed/end/step_dedicated_fans.rs. start() runs inline
 * (no hooks): a draw with no concession short-circuits to NEXT_STEP with a plain ReportDedicatedFans and
 * both modifiers left at 0 (no dice); otherwise it rolls homeDie then awayDie (6 normally, 3 for an
 * illegally-conceding team) and applies modifier(roll, dedicatedFans, winning, conceded). Dice are
 * scripted in fixed home-then-away order via installScriptedDice. The private modifier() branch matrix
 * is unit-tested on the Rust side directly; here we drive it through start() with scripted rolls.
 */
public class StepDedicatedFansFixtureTest {

	private GameState gameState;
	private Game game;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
		game = gameState.getGame();
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.DEDICATED_FANS);
	}

	// rust: draw_sets_no_modifier — tied score, no concession → NEXT_STEP, no dice, modifiers stay 0
	@Test
	public void drawNoConcessionLeavesModifiersZero() {
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(newStep()));
		assertEquals(0, game.getGameResult().getTeamResultHome().getDedicatedFansModifier());
		assertEquals(0, game.getGameResult().getTeamResultAway().getDedicatedFansModifier());
	}

	// rust: winning_team_gets_positive_or_zero_modifier — home leads on score; winner rolls at/above its
	// dedicated-fans value → +1, the non-winner rolls below its value → -1.
	@Test
	public void higherScoreWinnerGetsRollBasedModifiers() {
		game.getGameResult().getTeamResultHome().setScore(1);
		game.getTeamHome().setDedicatedFans(1);
		game.getTeamAway().setDedicatedFans(6);
		GameFixture.installScriptedDice(gameState, 6, 1); // homeDie then awayDie
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(newStep()));
		assertEquals(1, game.getGameResult().getTeamResultHome().getDedicatedFansModifier());
		assertEquals(-1, game.getGameResult().getTeamResultAway().getDedicatedFansModifier());
	}

	// rust: conceding_team_gets_negative_modifier — an illegally-conceding home team rolls a d3 (capped at
	// dedicatedFans-1, negated); the away team is treated as the winner.
	@Test
	public void concedingTeamGetsNegativeModifier() {
		game.getGameResult().getTeamResultHome().setConceded(true);
		game.getTeamHome().setDedicatedFans(3);
		game.getTeamAway().setDedicatedFans(1);
		GameFixture.installScriptedDice(gameState, 2, 6); // homeDie(=3) then awayDie(=6)
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(newStep()));
		assertEquals(-2, game.getGameResult().getTeamResultHome().getDedicatedFansModifier());
		assertEquals(1, game.getGameResult().getTeamResultAway().getDedicatedFansModifier());
	}
}
