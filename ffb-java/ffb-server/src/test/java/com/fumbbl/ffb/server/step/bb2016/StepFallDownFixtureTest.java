package com.fumbbl.ffb.server.step.bb2016;

import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.injury.injuryType.InjuryTypeDropDodge;
import com.fumbbl.ffb.server.injury.injuryType.InjuryTypeDropGFI;
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
import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/bb2016/step_fall_down.rs} (param + injury/blood-lust
 * subset). INJURY_TYPE and COORDINATE_FROM are stored via setParameter. The armour/injury dice are
 * preset via installScriptedDice (values do not affect these assertions). The publishes-injury-result
 * / publishes-end-turn / no-end-turn-pass-block / clears-move-squares / safe-pair-of-hands tests
 * inspect published params, turn mode, or move squares and are deferred.
 */
public class StepFallDownFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
		gameState.getGame().setHomePlaying(true);
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.MOVE);
	}

	private IStep fallStep() {
		IStep step = GameFixture.createStep(gameState, StepId.FALL_DOWN);
		step.setParameter(StepParameter.from(StepParameterKey.INJURY_TYPE, new InjuryTypeDropGFI()));
		GameFixture.installScriptedDice(gameState, 3, 4, 3, 4, 3, 4);
		return step;
	}

	// rust: set_parameter_injury_type_name_accepted
	@Test
	public void setParameterInjuryTypeAccepted() {
		assertTrue(GameFixture.createStep(gameState, StepId.FALL_DOWN)
			.setParameter(StepParameter.from(StepParameterKey.INJURY_TYPE, new InjuryTypeDropDodge())));
	}

	// rust: set_parameter_coordinate_from_accepted
	@Test
	public void setParameterCoordinateFromAccepted() {
		assertTrue(GameFixture.createStep(gameState, StepId.FALL_DOWN)
			.setParameter(StepParameter.from(StepParameterKey.COORDINATE_FROM, new FieldCoordinate(5, 5))));
	}

	// rust: unrecognised_parameter_returns_false
	@Test
	public void unrecognisedParameterReturnsFalse() {
		assertFalse(GameFixture.createStep(gameState, StepId.FALL_DOWN)
			.setParameter(StepParameter.from(StepParameterKey.END_TURN, true)));
	}

	// rust: start_returns_next_step
	@Test
	public void startReturnsNextStep() {
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(fallStep()));
	}

	// rust: blood_lust_sets_player_state_to_reserve
	@Test
	public void bloodLustSetsPlayerStateToReserve() {
		Game game = gameState.getGame();
		game.getActingPlayer().setSufferingBloodLust(true);
		GameFixture.startStep(fallStep());
		Player<?> player = game.getPlayerById("home1");
		assertEquals(PlayerState.RESERVE, game.getFieldModel().getPlayerState(player).getBase());
	}

	// rust: no_blood_lust_does_not_change_state_to_reserve
	@Test
	public void noBloodLustDoesNotChangeStateToReserve() {
		Game game = gameState.getGame();
		game.getActingPlayer().setSufferingBloodLust(false);
		GameFixture.startStep(fallStep());
		Player<?> player = game.getPlayerById("home1");
		assertNotEquals(PlayerState.RESERVE, game.getFieldModel().getPlayerState(player).getBase());
	}
}
