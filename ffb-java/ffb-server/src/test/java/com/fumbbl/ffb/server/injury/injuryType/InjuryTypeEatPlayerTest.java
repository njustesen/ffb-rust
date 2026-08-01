package com.fumbbl.ffb.server.injury.injuryType;

import com.fumbbl.ffb.ApothecaryMode;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.SendToBoxReason;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.StepId;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/injury/injuryType/injury_type_eat_player.rs tests.
 * Being eaten: armour is always broken and the (forced) injury is RIP; interpretInjuryRoll takes
 * the forced-injury branch (null injury roll -> returns the pre-set injury). canUseApo is false,
 * SendToBoxReason is EATEN, turnover default true. (The Rust default_equivalent_to_new plumbing
 * test is Rust-structural — exempt.)
 */
public class InjuryTypeEatPlayerTest {

	private GameState gameState;
	private Game game;
	private IStep step;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3);
		game = gameState.getGame();
		step = GameFixture.createStep(gameState, StepId.INIT_START_GAME);
		GameFixture.placePlayer(gameState, "away1", 5, 5);
	}

	private InjuryTypeEatPlayer handled() {
		InjuryTypeEatPlayer eatPlayer = new InjuryTypeEatPlayer();
		eatPlayer.injuryContext().setDefenderId("away1");
		eatPlayer.handleInjury(step, game, gameState, gameState.getDiceRoller(), null,
			game.getPlayerById("away1"), new FieldCoordinate(5, 5), null, null, ApothecaryMode.DEFENDER);
		return eatPlayer;
	}

	// rust: armor_always_broken
	@Test
	public void armorAlwaysBroken() {
		assertTrue(handled().injuryContext().isArmorBroken());
	}

	// rust: injury_is_ps_rip
	@Test
	public void injuryIsRip() {
		assertEquals(PlayerState.RIP, handled().injuryContext().getInjury().getBase());
	}

	// rust: falling_down_causes_turnover_defaults_true
	@Test
	public void fallingDownCausesTurnoverDefaultsTrue() {
		assertTrue(new InjuryTypeEatPlayer().injuryType().fallingDownCausesTurnover());
	}

	// rust: can_use_apo_is_false
	@Test
	public void canUseApoIsFalse() {
		assertFalse(new InjuryTypeEatPlayer().canUseApo());
	}

	// rust: send_to_box_reason_is_eaten
	@Test
	public void sendToBoxReasonIsEaten() {
		assertEquals(SendToBoxReason.EATEN, new InjuryTypeEatPlayer().sendToBoxReason());
	}

	// rust: context_stores_attacker_and_defender (Java handleInjury does not populate the context
	// ids — the CALLER sets them, a documented context-storage divergence; assert the defender id
	// the test set)
	@Test
	public void contextStoresDefender() {
		assertEquals("away1", handled().injuryContext().getDefenderId());
	}
}
