package com.fumbbl.ffb.server.step.action.pass;

import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.StepAction;
import com.fumbbl.ffb.server.step.StepId;
import com.fumbbl.ffb.server.step.StepParameter;
import com.fumbbl.ffb.server.step.StepParameterKey;
import com.fumbbl.ffb.server.step.StepParameterSet;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/step/action/pass/step_dispatch_passing.rs. The dispatch
 * reads game.getThrowerAction(): PASS/THROW_BOMB/DUMP_OFF → NEXT_STEP; HAIL_MARY_PASS/HAIL_MARY_BOMB →
 * GOTO_LABEL_ON_HAIL_MARY_PASS; HAND_OVER → GOTO_LABEL_ON_HAND_OVER; anything else → GOTO_LABEL_ON_END;
 * no thrower → waits (CONTINUE). The three goto labels are mandatory init params; CATCHER_ID is
 * accepted via setParameter.
 */
public class StepDispatchPassingFixtureTest {

	private GameState gameState;
	private Game game;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
		game = gameState.getGame();
		GameFixture.placePlayer(gameState, "home1", 5, 5);
	}

	private IStep newStep() {
		IStep step = GameFixture.createStep(gameState, StepId.DISPATCH_PASSING);
		StepParameterSet set = new StepParameterSet();
		set.add(StepParameter.from(StepParameterKey.GOTO_LABEL_ON_END, "end"));
		set.add(StepParameter.from(StepParameterKey.GOTO_LABEL_ON_HAIL_MARY_PASS, "hailmary"));
		set.add(StepParameter.from(StepParameterKey.GOTO_LABEL_ON_HAND_OVER, "handover"));
		step.init(set);
		return step;
	}

	private IStep withThrower(PlayerAction action) {
		game.setThrowerId("home1");
		game.setThrowerAction(action);
		return newStep();
	}

	// rust: pass_action_returns_next
	@Test
	public void passActionReturnsNext() {
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(withThrower(PlayerAction.PASS)));
	}

	// rust: throw_bomb_action_returns_next
	@Test
	public void throwBombActionReturnsNext() {
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(withThrower(PlayerAction.THROW_BOMB)));
	}

	// rust: dump_off_action_returns_next
	@Test
	public void dumpOffActionReturnsNext() {
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(withThrower(PlayerAction.DUMP_OFF)));
	}

	// rust: hail_mary_pass_gotos_hailmary_label
	@Test
	public void hailMaryPassGotosHailmaryLabel() {
		IStep step = withThrower(PlayerAction.HAIL_MARY_PASS);
		assertEquals(StepAction.GOTO_LABEL, GameFixture.startStep(step));
		assertEquals("hailmary", step.getResult().getNextActionParameter());
	}

	// rust: hail_mary_bomb_gotos_hailmary_label
	@Test
	public void hailMaryBombGotosHailmaryLabel() {
		IStep step = withThrower(PlayerAction.HAIL_MARY_BOMB);
		assertEquals(StepAction.GOTO_LABEL, GameFixture.startStep(step));
		assertEquals("hailmary", step.getResult().getNextActionParameter());
	}

	// rust: hand_over_gotos_handover_label
	@Test
	public void handOverGotosHandoverLabel() {
		IStep step = withThrower(PlayerAction.HAND_OVER);
		assertEquals(StepAction.GOTO_LABEL, GameFixture.startStep(step));
		assertEquals("handover", step.getResult().getNextActionParameter());
	}

	// rust: unknown_action_gotos_end_label
	@Test
	public void unknownActionGotosEndLabel() {
		IStep step = withThrower(PlayerAction.MOVE);
		assertEquals(StepAction.GOTO_LABEL, GameFixture.startStep(step));
		assertEquals("end", step.getResult().getNextActionParameter());
	}

	// rust: no_thrower_continues_waiting
	@Test
	public void noThrowerContinuesWaiting() {
		assertEquals(StepAction.CONTINUE, GameFixture.startStep(newStep()));
	}

	// rust: catcher_id_parameter_accepted
	@Test
	public void catcherIdParameterAccepted() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.CATCHER_ID, "home2")));
	}
}
