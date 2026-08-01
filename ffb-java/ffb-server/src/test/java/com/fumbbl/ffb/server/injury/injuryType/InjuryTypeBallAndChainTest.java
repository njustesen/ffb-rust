package com.fumbbl.ffb.server.injury.injuryType;

import com.fumbbl.ffb.ApothecaryMode;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.SendToBoxReason;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.RosterPlayer;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.StepId;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import java.util.Arrays;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/injury/injuryType/injury_type_ball_and_chain.rs tests.
 * Ball & Chain always breaks armour on knock-down; injury is rolled directly (with attacker
 * injury modifiers via findInjuryModifiers). SendToBoxReason is BALL_AND_CHAIN;
 * failedArmourPlacesProne stays at the base default true. (The Rust attacker_id_stored_in_context
 * test is a caller-populated context-storage divergence — Java handleInjury does not set the ids —
 * exempt.)
 */
public class InjuryTypeBallAndChainTest {

	private GameState gameState;
	private Game game;
	private IStep step;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3);
		game = gameState.getGame();
		step = GameFixture.createStep(gameState, StepId.INIT_START_GAME);
		GameFixture.placePlayer(gameState, "home1", 2, 2);
		GameFixture.placePlayer(gameState, "away1", 5, 5);
	}

	private RosterPlayer defender() {
		return (RosterPlayer) game.getPlayerById("away1");
	}

	private RosterPlayer attacker() {
		return (RosterPlayer) game.getPlayerById("home1");
	}

	private void handleInjury(InjuryTypeBallAndChain ballAndChain, RosterPlayer pAttacker) {
		ballAndChain.injuryContext().setDefenderId("away1");
		// variable modifiers (Mighty Blow value) resolve via the context attacker id
		if (pAttacker != null) {
			ballAndChain.injuryContext().setAttackerId(pAttacker.getId());
		}
		ballAndChain.handleInjury(step, game, gameState, gameState.getDiceRoller(), pAttacker, defender(),
			new FieldCoordinate(5, 5), null, null, ApothecaryMode.DEFENDER);
	}

	private boolean hasInjuryModifier(InjuryTypeBallAndChain ballAndChain, String namePart) {
		return Arrays.stream(ballAndChain.injuryContext().getInjuryModifiers())
			.anyMatch(m -> m.getName() != null && m.getName().contains(namePart));
	}

	// rust: mighty_blow_adds_injury_modifier
	@Test
	public void mightyBlowAddsInjuryModifier() {
		attacker().addSkill(GameFixture.skill(game, "Mighty Blow"));
		GameFixture.installScriptedDice(gameState, 3, 3);
		InjuryTypeBallAndChain ballAndChain = new InjuryTypeBallAndChain();
		handleInjury(ballAndChain, attacker());
		assertTrue(hasInjuryModifier(ballAndChain, "Mighty Blow"));
	}

	// rust: no_mighty_blow_no_injury_modifier
	@Test
	public void noMightyBlowNoInjuryModifier() {
		GameFixture.installScriptedDice(gameState, 3, 3);
		InjuryTypeBallAndChain ballAndChain = new InjuryTypeBallAndChain();
		handleInjury(ballAndChain, attacker());
		assertFalse(hasInjuryModifier(ballAndChain, "Mighty Blow"));
	}

	// rust: armor_always_broken
	@Test
	public void armorAlwaysBroken() {
		GameFixture.installScriptedDice(gameState, 3, 3);
		InjuryTypeBallAndChain ballAndChain = new InjuryTypeBallAndChain();
		handleInjury(ballAndChain, null);
		assertTrue(ballAndChain.injuryContext().isArmorBroken());
	}

	// rust: injury_is_set
	@Test
	public void injuryIsSet() {
		GameFixture.installScriptedDice(gameState, 3, 3);
		InjuryTypeBallAndChain ballAndChain = new InjuryTypeBallAndChain();
		handleInjury(ballAndChain, null);
		assertNotNull(ballAndChain.injuryContext().getInjury());
		assertNotEquals(PlayerState.PRONE, ballAndChain.injuryContext().getInjury().getBase());
	}

	// rust: failed_armour_places_prone_default_true
	@Test
	public void failedArmourPlacesProneDefaultTrue() {
		assertTrue(new InjuryTypeBallAndChain().injuryType().failedArmourPlacesProne());
	}

	// rust: send_to_box_reason_is_ball_and_chain
	@Test
	public void sendToBoxReasonIsBallAndChain() {
		assertEquals(SendToBoxReason.BALL_AND_CHAIN, new InjuryTypeBallAndChain().sendToBoxReason());
	}
}
