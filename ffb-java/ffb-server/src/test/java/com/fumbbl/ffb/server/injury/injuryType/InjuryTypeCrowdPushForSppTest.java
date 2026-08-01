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
 * Mirror of ffb-rust crates/ffb-engine/src/injury/injuryType/injury_type_crowd_push_for_spp.rs
 * tests. Crowd-base injury (armour always broken; a non-KO/non-casualty result becomes RESERVE)
 * that is worth SPPs and caused by opponent, with no turnover. SendToBoxReason is CROWD_PUSHED.
 * (The Rust context_stores_defender_id test is caller-populated — exempt; default_equivalent_to_new
 * is Rust-structural — exempt.)
 */
public class InjuryTypeCrowdPushForSppTest {

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

	private InjuryTypeCrowdPushForSpp handled() {
		InjuryTypeCrowdPushForSpp crowdPush = new InjuryTypeCrowdPushForSpp();
		crowdPush.injuryContext().setDefenderId("away1");
		crowdPush.handleInjury(step, game, gameState, gameState.getDiceRoller(), null,
			game.getPlayerById("away1"), new FieldCoordinate(5, 5), null, null, ApothecaryMode.DEFENDER);
		return crowdPush;
	}

	// rust: armor_always_broken
	@Test
	public void armorAlwaysBroken() {
		GameFixture.installScriptedDice(gameState, 1, 1);
		assertTrue(handled().injuryContext().isArmorBroken());
	}

	// rust: injury_is_reserve_or_ko_or_casualty (deterministic Java version: stunned -> RESERVE)
	@Test
	public void injuryIsReserveWhenNotKoOrCasualty() {
		GameFixture.installScriptedDice(gameState, 1, 1); // total 2 -> stunned -> RESERVE
		assertEquals(PlayerState.RESERVE, handled().injuryContext().getInjury().getBase());
	}

	// rust: does_not_cause_turnover
	@Test
	public void doesNotCauseTurnover() {
		assertFalse(new InjuryTypeCrowdPushForSpp().injuryType().fallingDownCausesTurnover());
	}

	// rust: is_worth_spps / is_caused_by_opponent traits — the defining difference from base CrowdPush
	@Test
	public void isWorthSppsAndCausedByOpponent() {
		InjuryTypeCrowdPushForSpp crowdPush = new InjuryTypeCrowdPushForSpp();
		assertTrue(crowdPush.injuryType().isWorthSpps());
		assertTrue(crowdPush.injuryType().isCausedByOpponent());
	}

	// rust: send_to_box_reason_is_crowd_pushed
	@Test
	public void sendToBoxReasonIsCrowdPushed() {
		assertEquals(SendToBoxReason.CROWD_PUSHED, new InjuryTypeCrowdPushForSpp().sendToBoxReason());
	}
}
