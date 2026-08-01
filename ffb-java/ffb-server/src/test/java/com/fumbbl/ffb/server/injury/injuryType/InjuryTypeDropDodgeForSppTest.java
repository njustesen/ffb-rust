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
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/injury/injuryType/injury_type_drop_dodge_for_spp.rs
 * tests. Java's constructor requires the arm-bar/diving-tackle player, and handleInjury credits
 * it as the attacker unconditionally.
 */
public class InjuryTypeDropDodgeForSppTest {

	private GameState gameState;
	private Game game;
	private IStep step;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3);
		game = gameState.getGame();
		step = GameFixture.createStep(gameState, StepId.INIT_START_GAME);
		GameFixture.placePlayer(gameState, "away1", 5, 5);  // defender (the dodging player)
		GameFixture.placePlayer(gameState, "away2", 10, 10); // arm-bar player
	}

	private RosterPlayer defender() {
		return (RosterPlayer) game.getPlayerById("away1");
	}

	private RosterPlayer armBarPlayer() {
		return (RosterPlayer) game.getPlayerById("away2");
	}

	private InjuryTypeDropDodgeForSpp newInjuryType() {
		return new InjuryTypeDropDodgeForSpp(armBarPlayer());
	}

	private void handleInjury(InjuryTypeDropDodgeForSpp dropDodge) {
		dropDodge.injuryContext().setDefenderId("away1");
		dropDodge.handleInjury(step, game, gameState, gameState.getDiceRoller(), null, defender(),
			new FieldCoordinate(5, 5), null, null, ApothecaryMode.DEFENDER);
	}

	private boolean hasInjuryModifier(InjuryTypeDropDodgeForSpp dropDodge, String name) {
		return Arrays.stream(dropDodge.injuryContext().getInjuryModifiers())
			.anyMatch(m -> name.equals(m.getName()));
	}

	// rust: armor_save_results_in_prone
	@Test
	public void armorSaveResultsInProne() {
		defender().setArmour(13);
		GameFixture.installScriptedDice(gameState, 1, 1);
		InjuryTypeDropDodgeForSpp dropDodge = newInjuryType();
		handleInjury(dropDodge);
		assertEquals(PlayerState.PRONE, dropDodge.injuryContext().getInjury().getBase());
	}

	// rust: armor_break_results_in_injury_roll
	@Test
	public void armorBreakResultsInInjuryRoll() {
		defender().setArmour(2);
		GameFixture.installScriptedDice(gameState, 6, 6, 1, 1);
		InjuryTypeDropDodgeForSpp dropDodge = newInjuryType();
		handleInjury(dropDodge);
		assertTrue(dropDodge.injuryContext().isArmorBroken());
		assertNotEquals(PlayerState.PRONE, dropDodge.injuryContext().getInjury().getBase());
	}

	// rust: falling_down_causes_turnover
	@Test
	public void fallingDownCausesTurnover() {
		assertTrue(newInjuryType().injuryType().fallingDownCausesTurnover());
	}

	// rust: is_worth_spps_is_true
	@Test
	public void isWorthSppsIsTrue() {
		assertTrue(newInjuryType().injuryType().isWorthSpps());
	}

	// rust: is_caused_by_opponent_is_true
	@Test
	public void isCausedByOpponentIsTrue() {
		assertTrue(newInjuryType().injuryType().isCausedByOpponent());
	}

	// rust: send_to_box_reason_is_dodge_fail
	@Test
	public void sendToBoxReasonIsDodgeFail() {
		assertEquals(SendToBoxReason.DODGE_FAIL, newInjuryType().sendToBoxReason());
	}

	// rust: attacker_id_credited_to_arm_bar_player_not_handle_injury_attacker
	@Test
	public void attackerIdCreditedToArmBarPlayerNotHandleInjuryAttacker() {
		defender().setArmour(13);
		GameFixture.installScriptedDice(gameState, 1, 1);
		InjuryTypeDropDodgeForSpp dropDodge = newInjuryType();
		dropDodge.injuryContext().setDefenderId("away1");
		// pass a DIFFERENT attacker: the arm-bar player must still be credited
		dropDodge.handleInjury(step, game, gameState, gameState.getDiceRoller(),
			game.getPlayerById("home1"), defender(), new FieldCoordinate(5, 5), null, null,
			ApothecaryMode.DEFENDER);
		assertEquals("away2", dropDodge.injuryContext().getAttackerId());
	}

	// rust: pre_broken_armor_skips_armor_roll_goes_to_injury
	@Test
	public void preBrokenArmorSkipsArmorRollGoesToInjury() {
		defender().setArmour(7);
		GameFixture.installScriptedDice(gameState, 1, 1); // injury dice only
		InjuryTypeDropDodgeForSpp dropDodge = newInjuryType();
		dropDodge.injuryContext().setArmorBroken(true);
		handleInjury(dropDodge);
		assertTrue(dropDodge.injuryContext().isArmorBroken());
		assertNotEquals(PlayerState.PRONE, dropDodge.injuryContext().getInjury().getBase());
	}

	// rust: mighty_blow_adds_injury_modifier
	// (Java sources the injury modifiers from the pAttacker PARAMETER — the arm-bar player only
	// contributes its affectsEitherArmourOrInjuryOnDodge skill, mirroring the Rust setup where
	// Mighty Blow sits on the handle_injury attacker.)
	@Test
	public void mightyBlowAddsInjuryModifier() {
		GameFixture.placePlayer(gameState, "home1", 2, 2);
		((RosterPlayer) game.getPlayerById("home1")).addSkill(GameFixture.skill(game, "Mighty Blow"));
		defender().setArmour(2);
		GameFixture.installScriptedDice(gameState, 6, 6, 1, 1);
		InjuryTypeDropDodgeForSpp dropDodge = newInjuryType();
		dropDodge.injuryContext().setDefenderId("away1");
		dropDodge.handleInjury(step, game, gameState, gameState.getDiceRoller(),
			game.getPlayerById("home1"), defender(), new FieldCoordinate(5, 5), null, null,
			ApothecaryMode.DEFENDER);
		assertTrue(hasInjuryModifier(dropDodge, "Mighty Blow"));
	}

	// rust: no_mighty_blow_no_injury_modifier
	@Test
	public void noMightyBlowNoInjuryModifier() {
		defender().setArmour(2);
		GameFixture.installScriptedDice(gameState, 6, 6, 1, 1);
		InjuryTypeDropDodgeForSpp dropDodge = newInjuryType();
		handleInjury(dropDodge);
		assertFalse(hasInjuryModifier(dropDodge, "Mighty Blow"));
	}
}
