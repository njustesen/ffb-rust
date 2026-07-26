package com.fumbbl.ffb.server.step.bb2016;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.TurnMode;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.StepAction;
import com.fumbbl.ffb.server.step.StepId;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;

/**
 * Mirrors the Rust tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/bb2016/step_end_turn.rs}.
 * Only the regular-turn state transition is a clean 1:1 port: the Rust set_parameter tests use
 * step fields (touchdown_player_id/end_game/new_half) that the Java StepEndTurn does not accept via
 * setParameter (different StepParameterKey scheme; command-driven), and the touchdown/half/kickoff
 * transitions + argue-the-call / use-bribe command tests need deep game state — all deferred.
 */
public class StepEndTurnFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.END_TURN);
	}

	// rust: regular_turn_flips_home_playing_and_increments_turn_nr
	@Test
	public void regularTurnFlipsHomePlayingAndIncrementsTurnNr() {
		Game game = gameState.getGame();
		game.setTurnMode(TurnMode.REGULAR);
		game.setHomePlaying(true);
		game.getTurnDataHome().setTurnNr(3);
		game.getTurnDataAway().setTurnNr(3);
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(newStep()));
		assertFalse(game.isHomePlaying());
		assertEquals(4, game.getTurnDataAway().getTurnNr());
		assertEquals(3, game.getTurnDataHome().getTurnNr());
	}
}
