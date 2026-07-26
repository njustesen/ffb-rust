package com.fumbbl.ffb.server.step.bb2016.ttm;

import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.StepId;
import com.fumbbl.ffb.server.step.StepParameter;
import com.fumbbl.ffb.server.step.StepParameterKey;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirrors the Rust tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/bb2016/ttm/step_fumble_ttm_pass.rs} (placement subset).
 * THROWN_PLAYER_ID / THROWN_PLAYER_COORDINATE / THROWN_PLAYER_STATE are stored via setParameter. The
 * placement + defender-clear runs only when all three are set and the state's raw id is &gt; 0 (the
 * Java guard checks the full encoded int, not just the low-byte base). always_publishes_coordinate_reset
 * inspects a published parameter and is deferred.
 */
public class StepFumbleTtmPassFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
		gameState.getGame().setHomePlaying(true);
		GameFixture.placePlayer(gameState, "away1", 3, 3);
		gameState.getGame().setDefenderId("away1");
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.FUMBLE_TTM_PASS);
	}

	private IStep stepWith(PlayerState state) {
		IStep step = newStep();
		step.setParameter(StepParameter.from(StepParameterKey.THROWN_PLAYER_ID, "away1"));
		step.setParameter(StepParameter.from(StepParameterKey.THROWN_PLAYER_COORDINATE, new FieldCoordinate(10, 7)));
		step.setParameter(StepParameter.from(StepParameterKey.THROWN_PLAYER_STATE, state));
		return step;
	}

	// rust: no_thrown_player_clears_nothing
	@Test
	public void noThrownPlayerClearsNothing() {
		GameFixture.startStep(newStep());
		assertNotNull(gameState.getGame().getDefenderId());
	}

	// rust: unknown_state_zero_skips_placement
	@Test
	public void unknownStateZeroSkipsPlacement() {
		GameFixture.startStep(stepWith(new PlayerState(0)));
		assertNotNull(gameState.getGame().getDefenderId());
	}

	// rust: nonzero_raw_id_with_zero_base_still_places
	@Test
	public void nonzeroRawIdWithZeroBaseStillPlaces() {
		GameFixture.startStep(stepWith(new PlayerState(0x100)));
		Game game = gameState.getGame();
		assertEquals(new FieldCoordinate(10, 7), game.getFieldModel().getPlayerCoordinate(game.getPlayerById("away1")));
		assertNull(game.getDefenderId());
	}

	// rust: with_valid_state_places_and_clears_defender
	@Test
	public void withValidStatePlacesAndClearsDefender() {
		GameFixture.startStep(stepWith(new PlayerState(PlayerState.PRONE)));
		Game game = gameState.getGame();
		Player<?> p = game.getPlayerById("away1");
		assertEquals(new FieldCoordinate(10, 7), game.getFieldModel().getPlayerCoordinate(p));
		assertNull(game.getDefenderId());
		assertEquals(PlayerState.PRONE, game.getFieldModel().getPlayerState(p).getBase());
	}
}
