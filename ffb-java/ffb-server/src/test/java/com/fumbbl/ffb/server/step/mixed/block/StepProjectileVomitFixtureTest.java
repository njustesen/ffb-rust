package com.fumbbl.ffb.server.step.mixed.block;

import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.StepAction;
import com.fumbbl.ffb.server.step.StepId;
import com.fumbbl.ffb.server.step.StepParameter;
import com.fumbbl.ffb.server.step.StepParameterKey;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/step/mixed/block/step_projectile_vomit.rs (guard subset).
 * Without the vomit-armour-roll property (or with usingVomit false), the step passes through to
 * NEXT_STEP. The vomit roll / report / reroll-offer tests are dice/command-driven; USING_VOMIT +
 * GOTO_LABEL_* are init-consumed (setParameter twins exempt); no_acting_player is Rust-defensive.
 */
public class StepProjectileVomitFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.BLOCK);
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.PROJECTILE_VOMIT);
	}

	// rust: without_vomit_skill_returns_next
	@Test
	public void withoutVomitSkillReturnsNext() {
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(newStep()));
	}

	// unrecognised parameter returns false
	@Test
	public void unknownParameterReturnsFalse() {
		assertFalse(newStep().setParameter(StepParameter.from(StepParameterKey.END_TURN, true)));
	}
}
