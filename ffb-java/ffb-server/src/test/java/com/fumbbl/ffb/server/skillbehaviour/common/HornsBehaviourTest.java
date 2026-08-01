package com.fumbbl.ffb.server.skillbehaviour.common;

import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.RosterPlayer;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.step.StepAction;
import com.fumbbl.ffb.server.step.action.block.StepHorns;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/skill_behaviour/common/horns_behaviour.rs tests
 * (portable subset — registry plumbing exempt). Horns: +1 strength when BLITZING; the behaviour
 * marks the skill used and adds a report only while blitzing, then advances to the next step.
 * StepHorns.StepState is a non-static inner class, so usingHorns is asserted via the observable
 * markSkillUsed / report side-effects, mirroring the Rust hook-state assertions.
 */
public class HornsBehaviourTest {

	private GameState gameState;
	private Game game;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
		game = gameState.getGame();
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		((RosterPlayer) game.getPlayerById("home1")).addSkill(GameFixture.skill(game, "Horns"));
	}

	private StepHorns runHorns(PlayerAction action) {
		GameFixture.setActingPlayer(gameState, "home1", action);
		StepHorns step = new StepHorns(gameState);
		step.start();
		return step;
	}

	private boolean skillUsed() {
		return game.getActingPlayer().isSkillUsed(GameFixture.skill(game, "Horns"));
	}

	// rust: modifier_marks_skill_used_when_blitzing (+ sets_using_horns_true_when_blitzing)
	@Test
	public void marksSkillUsedWhenBlitzing() {
		runHorns(PlayerAction.BLITZ);
		assertTrue(skillUsed());
	}

	// rust: modifier_adds_report_when_blitzing
	@Test
	public void addsReportWhenBlitzing() {
		StepHorns step = runHorns(PlayerAction.BLITZ);
		assertTrue(step.getResult().getReportList().size() > 0);
	}

	// rust: modifier_sets_using_horns_false_when_blocking (skill not used, no report)
	@Test
	public void doesNotUseHornsWhenBlocking() {
		StepHorns step = runHorns(PlayerAction.BLOCK);
		assertFalse(skillUsed());
		assertEquals(0, step.getResult().getReportList().size());
	}

	// rust: modifier_advances_to_next_step (the hook always sets NEXT_STEP)
	@Test
	public void advancesToNextStep() {
		StepHorns step = runHorns(PlayerAction.BLITZ);
		assertEquals(StepAction.NEXT_STEP, step.getResult().getNextAction());
	}
}
