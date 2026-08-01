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
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/injury/injuryType/injury_type_throw_a_rock.rs tests.
 */
public class InjuryTypeThrowARockTest {

	private GameState gameState;
	private Game game;
	private IStep step;

	@BeforeEach
	void setUp() {
		init(GameFixture.createGameState(3));
	}

	private void init(GameState gs) {
		gameState = gs;
		game = gameState.getGame();
		step = GameFixture.createStep(gameState, StepId.INIT_START_GAME);
		GameFixture.placePlayer(gameState, "home1", 2, 2); // attacker
		GameFixture.placePlayer(gameState, "away1", 5, 5); // defender
	}

	private RosterPlayer attacker() {
		return (RosterPlayer) game.getPlayerById("home1");
	}

	private RosterPlayer defender() {
		return (RosterPlayer) game.getPlayerById("away1");
	}

	private void handleInjury(InjuryTypeThrowARock injuryType) {
		injuryType.injuryContext().setAttackerId("home1");
		injuryType.injuryContext().setDefenderId("away1");
		injuryType.handleInjury(step, game, gameState, gameState.getDiceRoller(), attacker(), defender(),
			new FieldCoordinate(5, 5), null, null, ApothecaryMode.DEFENDER);
	}

	private boolean hasInjuryModifier(InjuryTypeThrowARock injuryType, String namePart) {
		return Arrays.stream(injuryType.injuryContext().getInjuryModifiers())
			.anyMatch(m -> m.getName() != null && m.getName().contains(namePart));
	}

	// rust: armor_is_always_broken
	@Test
	public void armorIsAlwaysBroken() {
		defender().setArmour(13);
		GameFixture.installScriptedDice(gameState, 1, 1); // injury dice only
		InjuryTypeThrowARock rock = new InjuryTypeThrowARock();
		handleInjury(rock);
		assertTrue(rock.injuryContext().isArmorBroken());
	}

	// rust: injury_is_set
	@Test
	public void injuryIsSet() {
		GameFixture.installScriptedDice(gameState, 1, 1);
		InjuryTypeThrowARock rock = new InjuryTypeThrowARock();
		handleInjury(rock);
		assertNotNull(rock.injuryContext().getInjury());
	}

	// rust: causes_turnover_by_default
	@Test
	public void causesTurnoverByDefault() {
		assertTrue(new InjuryTypeThrowARock().injuryType().fallingDownCausesTurnover());
	}

	// rust: send_to_box_reason_is_hit_by_rock
	@Test
	public void sendToBoxReasonIsHitByRock() {
		assertEquals(SendToBoxReason.HIT_BY_ROCK, new InjuryTypeThrowARock().sendToBoxReason());
	}

	// rust: mighty_blow_adds_injury_modifier
	@Test
	public void mightyBlowAddsInjuryModifier() {
		attacker().addSkill(GameFixture.skill(game, "Mighty Blow"));
		GameFixture.installScriptedDice(gameState, 1, 1);
		InjuryTypeThrowARock rock = new InjuryTypeThrowARock();
		handleInjury(rock);
		assertTrue(hasInjuryModifier(rock, "Mighty Blow"));
	}

	// rust: no_mighty_blow_no_injury_modifier
	@Test
	public void noMightyBlowNoInjuryModifier() {
		GameFixture.installScriptedDice(gameState, 1, 1);
		InjuryTypeThrowARock rock = new InjuryTypeThrowARock();
		handleInjury(rock);
		assertFalse(hasInjuryModifier(rock, "Mighty Blow"));
	}
}
