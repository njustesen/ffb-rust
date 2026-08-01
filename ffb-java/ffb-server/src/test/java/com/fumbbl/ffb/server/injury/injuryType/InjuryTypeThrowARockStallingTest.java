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
 * Mirror of ffb-rust crates/ffb-engine/src/injury/injuryType/injury_type_throw_a_rock_stalling.rs tests.
 */
public class InjuryTypeThrowARockStallingTest {

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

	private void handleInjury(InjuryTypeThrowARockStalling injuryType) {
		injuryType.injuryContext().setAttackerId("home1");
		injuryType.injuryContext().setDefenderId("away1");
		injuryType.handleInjury(step, game, gameState, gameState.getDiceRoller(), attacker(), defender(),
			new FieldCoordinate(5, 5), null, null, ApothecaryMode.DEFENDER);
	}

	private boolean hasInjuryModifier(InjuryTypeThrowARockStalling injuryType, String namePart) {
		return Arrays.stream(injuryType.injuryContext().getInjuryModifiers())
			.anyMatch(m -> m.getName() != null && m.getName().contains(namePart));
	}

	// rust: armor_save_results_in_prone
	@Test
	public void armorSaveResultsInProne() {
		defender().setArmour(13);
		GameFixture.installScriptedDice(gameState, 1, 1);
		InjuryTypeThrowARockStalling rock = new InjuryTypeThrowARockStalling();
		handleInjury(rock);
		assertEquals(PlayerState.PRONE, rock.injuryContext().getInjury().getBase());
	}

	// rust: armor_break_results_in_injury_roll
	@Test
	public void armorBreakResultsInInjuryRoll() {
		defender().setArmour(2);
		GameFixture.installScriptedDice(gameState, 6, 6, 1, 1);
		InjuryTypeThrowARockStalling rock = new InjuryTypeThrowARockStalling();
		handleInjury(rock);
		assertTrue(rock.injuryContext().isArmorBroken());
		assertNotEquals(PlayerState.PRONE, rock.injuryContext().getInjury().getBase());
	}

	// rust: causes_turnover_by_default
	@Test
	public void causesTurnoverByDefault() {
		assertTrue(new InjuryTypeThrowARockStalling().injuryType().fallingDownCausesTurnover());
	}

	// rust: send_to_box_reason_is_hit_by_rock
	@Test
	public void sendToBoxReasonIsHitByRock() {
		assertEquals(SendToBoxReason.HIT_BY_ROCK, new InjuryTypeThrowARockStalling().sendToBoxReason());
	}

	// rust: mighty_blow_adds_injury_modifier
	@Test
	public void mightyBlowAddsInjuryModifier() {
		attacker().addSkill(GameFixture.skill(game, "Mighty Blow"));
		defender().setArmour(2);
		GameFixture.installScriptedDice(gameState, 6, 6, 1, 1);
		InjuryTypeThrowARockStalling rock = new InjuryTypeThrowARockStalling();
		handleInjury(rock);
		assertTrue(hasInjuryModifier(rock, "Mighty Blow"));
	}

	// rust: no_mighty_blow_no_injury_modifier
	@Test
	public void noMightyBlowNoInjuryModifier() {
		defender().setArmour(2);
		GameFixture.installScriptedDice(gameState, 6, 6, 1, 1);
		InjuryTypeThrowARockStalling rock = new InjuryTypeThrowARockStalling();
		handleInjury(rock);
		assertFalse(hasInjuryModifier(rock, "Mighty Blow"));
	}
}
