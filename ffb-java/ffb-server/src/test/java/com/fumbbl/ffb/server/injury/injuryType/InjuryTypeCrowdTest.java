package com.fumbbl.ffb.server.injury.injuryType;

import com.fumbbl.ffb.ApothecaryMode;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.RosterPlayer;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.StepId;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import java.util.Arrays;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/injury/injuryType/injury_type_crowd.rs tests —
 * the abstract InjuryTypeCrowd base, driven through the concrete InjuryTypeCrowdPush.
 */
public class InjuryTypeCrowdTest {

	private GameState gameState;
	private Game game;
	private IStep step;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3);
		game = gameState.getGame();
		step = GameFixture.createStep(gameState, StepId.INIT_START_GAME);
		GameFixture.placePlayer(gameState, "home1", 2, 2); // attacker
		GameFixture.placePlayer(gameState, "away1", 5, 5); // defender
	}

	private void handleInjury(InjuryTypeCrowdPush crowdPush) {
		crowdPush.injuryContext().setAttackerId("home1");
		crowdPush.injuryContext().setDefenderId("away1");
		crowdPush.handleInjury(step, game, gameState, gameState.getDiceRoller(),
			game.getPlayerById("home1"), game.getPlayerById("away1"),
			new FieldCoordinate(5, 5), null, null, ApothecaryMode.DEFENDER);
	}

	private boolean hasInjuryModifier(InjuryTypeCrowdPush crowdPush, String name) {
		return Arrays.stream(crowdPush.injuryContext().getInjuryModifiers())
			.anyMatch(m -> name.equals(m.getName()));
	}

	// rust: sets_armor_broken_true
	@Test
	public void setsArmorBrokenTrue() {
		GameFixture.installScriptedDice(gameState, 1, 1);
		InjuryTypeCrowdPush crowdPush = new InjuryTypeCrowdPush();
		handleInjury(crowdPush);
		assertTrue(crowdPush.injuryContext().isArmorBroken());
	}

	// rust: injury_is_set_after_call
	@Test
	public void injuryIsSetAfterCall() {
		GameFixture.installScriptedDice(gameState, 1, 1);
		InjuryTypeCrowdPush crowdPush = new InjuryTypeCrowdPush();
		handleInjury(crowdPush);
		assertNotNull(crowdPush.injuryContext().getInjury());
	}

	// rust: mighty_blow_adds_injury_modifier
	@Test
	public void mightyBlowAddsInjuryModifier() {
		((RosterPlayer) game.getPlayerById("home1")).addSkill(GameFixture.skill(game, "Mighty Blow"));
		GameFixture.installScriptedDice(gameState, 1, 1);
		InjuryTypeCrowdPush crowdPush = new InjuryTypeCrowdPush();
		handleInjury(crowdPush);
		assertTrue(hasInjuryModifier(crowdPush, "Mighty Blow"));
	}

	// rust: no_mighty_blow_no_injury_modifier
	@Test
	public void noMightyBlowNoInjuryModifier() {
		GameFixture.installScriptedDice(gameState, 1, 1);
		InjuryTypeCrowdPush crowdPush = new InjuryTypeCrowdPush();
		handleInjury(crowdPush);
		assertFalse(hasInjuryModifier(crowdPush, "Mighty Blow"));
	}
}
