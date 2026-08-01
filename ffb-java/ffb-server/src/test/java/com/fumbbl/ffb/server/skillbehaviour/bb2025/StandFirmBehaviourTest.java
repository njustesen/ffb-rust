package com.fumbbl.ffb.server.skillbehaviour.bb2025;

import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.RosterPlayer;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.model.StepModifier;
import com.fumbbl.ffb.server.step.bb2025.block.StepPushback;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import java.util.HashMap;
import java.util.Stack;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/skill_behaviour/bb2025/stand_firm_behaviour.rs tests
 * (portable subset — registry plumbing exempt; Rust headless auto-decline for undecided is a
 * documented divergence, Java shows the skill-use dialog).
 */
public class StandFirmBehaviourTest {

	private GameState gameState;
	private Game game;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2025);
		game = gameState.getGame();
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		GameFixture.placePlayer(gameState, "away1", 10, 10);
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.MOVE);
	}

	private StepPushback.StepState newState() {
		StepPushback.StepState state = new StepPushback.StepState();
		state.defender = game.getPlayerById("away1");
		state.standingFirm = new HashMap<>();
		state.pushbackStack = new Stack<>();
		state.oldDefenderState = game.getFieldModel().getPlayerState(state.defender);
		return state;
	}

	@SuppressWarnings({"unchecked", "rawtypes"})
	private boolean executeHook(StepPushback.StepState state) {
		StepPushback step = new StepPushback(gameState);
		StandFirmBehaviour behaviour = new StandFirmBehaviour();
		behaviour.skill = (com.fumbbl.ffb.skill.common.StandFirm) GameFixture.skill(game, "Stand Firm");
		StepModifier modifier = behaviour.getStepModifiers().get(0);
		return modifier.handleExecuteStepHook(step, state);
	}

	private void addStandFirm() {
		((RosterPlayer) game.getPlayerById("away1")).addSkill(GameFixture.skill(game, "Stand Firm"));
	}

	// rust: no_stand_firm_skill_returns_false
	@Test
	public void noStandFirmSkillReturnsFalse() {
		StepPushback.StepState state = newState();
		assertFalse(executeHook(state));
		assertFalse(state.doPush);
	}

	// rust: stand_firm_not_decided_headless_auto_declines (Java variant: shows skill-use dialog)
	@Test
	public void standFirmNotDecidedShowsDialog() {
		addStandFirm();
		StepPushback.StepState state = newState();
		assertTrue(executeHook(state));
		assertNotNull(game.getDialogParameter());
		assertFalse(state.doPush);
	}

	// rust: stand_firm_accepted_cancels_push
	@Test
	public void standFirmAcceptedCancelsPush() {
		addStandFirm();
		StepPushback.StepState state = newState();
		state.standingFirm.put("away1", true);
		assertTrue(executeHook(state));
		assertTrue(state.doPush);
		assertTrue(state.pushbackStack.isEmpty());
	}

	// rust: stand_firm_declined_returns_false
	@Test
	public void standFirmDeclinedReturnsFalse() {
		addStandFirm();
		StepPushback.StepState state = newState();
		state.standingFirm.put("away1", false);
		assertFalse(executeHook(state));
		assertFalse(state.doPush);
	}

	// rust: stand_firm_rooted_auto_stands_firm
	@Test
	public void standFirmRootedAutoStandsFirm() {
		addStandFirm();
		PlayerState ps = game.getFieldModel().getPlayerState(game.getPlayerById("away1"));
		game.getFieldModel().setPlayerState(game.getPlayerById("away1"), ps.changeRooted(true));
		StepPushback.StepState state = newState();
		assertTrue(executeHook(state));
		assertTrue(state.doPush);
	}

	// rust: stand_firm_with_no_tacklezone_auto_declines
	@Test
	public void standFirmWithNoTacklezoneAutoDeclines() {
		addStandFirm();
		StepPushback.StepState state = newState();
		state.oldDefenderState = state.oldDefenderState.changeConfused(true);
		assertFalse(executeHook(state));
		assertFalse(state.doPush);
	}
}
