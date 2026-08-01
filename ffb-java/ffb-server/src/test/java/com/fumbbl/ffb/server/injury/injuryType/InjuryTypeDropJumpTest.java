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
 * Mirror of ffb-rust crates/ffb-engine/src/injury/injuryType/injury_type_drop_jump.rs tests.
 * A failed jump/leap: armour save leaves the player prone, a break rolls injury with attacker
 * modifiers (Mighty Blow). SendToBoxReason is JUMP_FAIL, turnover default true. (The Rust
 * default_equivalent_to_new plumbing test is Rust-structural — exempt.)
 */
public class InjuryTypeDropJumpTest {

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

	private void handleInjury(InjuryTypeDropJump dropJump, RosterPlayer pAttacker) {
		dropJump.injuryContext().setDefenderId("away1");
		// getInjuryModifierTotal() resolves variable modifiers (e.g. Mighty Blow's value) via the
		// context attacker/defender ids, so a modifier-bearing attacker must be recorded here.
		if (pAttacker != null) {
			dropJump.injuryContext().setAttackerId(pAttacker.getId());
		}
		dropJump.handleInjury(step, game, gameState, gameState.getDiceRoller(), pAttacker, defender(),
			new FieldCoordinate(5, 5), null, null, ApothecaryMode.DEFENDER);
	}

	private boolean hasInjuryModifier(InjuryTypeDropJump dropJump, String namePart) {
		return Arrays.stream(dropJump.injuryContext().getInjuryModifiers())
			.anyMatch(m -> m.getName() != null && m.getName().contains(namePart));
	}

	// rust: armor_save_results_in_prone
	@Test
	public void armorSaveResultsInProne() {
		GameFixture.installScriptedDice(gameState, 1, 1);
		InjuryTypeDropJump dropJump = new InjuryTypeDropJump();
		handleInjury(dropJump, null);
		assertFalse(dropJump.injuryContext().isArmorBroken());
		assertEquals(PlayerState.PRONE, dropJump.injuryContext().getInjury().getBase());
	}

	// rust: armor_break_results_in_injury_roll
	@Test
	public void armorBreakResultsInInjuryRoll() {
		GameFixture.installScriptedDice(gameState, 6, 6, 1, 1);
		InjuryTypeDropJump dropJump = new InjuryTypeDropJump();
		handleInjury(dropJump, null);
		assertTrue(dropJump.injuryContext().isArmorBroken());
		assertNotEquals(PlayerState.PRONE, dropJump.injuryContext().getInjury().getBase());
	}

	// rust: causes_turnover
	@Test
	public void causesTurnover() {
		assertTrue(new InjuryTypeDropJump().injuryType().fallingDownCausesTurnover());
	}

	// rust: send_to_box_reason_is_jump_fail
	@Test
	public void sendToBoxReasonIsJumpFail() {
		assertEquals(SendToBoxReason.JUMP_FAIL, new InjuryTypeDropJump().sendToBoxReason());
	}

	// rust: pre_broken_skips_armor_roll (armour already broken -> straight to injury roll)
	@Test
	public void preBrokenSkipsArmorRoll() {
		GameFixture.installScriptedDice(gameState, 1, 1);
		InjuryTypeDropJump dropJump = new InjuryTypeDropJump();
		dropJump.injuryContext().setArmorBroken(true);
		handleInjury(dropJump, null);
		assertTrue(dropJump.injuryContext().isArmorBroken());
		assertNotEquals(PlayerState.PRONE, dropJump.injuryContext().getInjury().getBase());
	}

	// rust: mighty_blow_adds_injury_modifier
	@Test
	public void mightyBlowAddsInjuryModifier() {
		attacker().addSkill(GameFixture.skill(game, "Mighty Blow"));
		GameFixture.installScriptedDice(gameState, 6, 6, 1, 1);
		InjuryTypeDropJump dropJump = new InjuryTypeDropJump();
		handleInjury(dropJump, attacker());
		assertTrue(dropJump.injuryContext().isArmorBroken());
		assertTrue(hasInjuryModifier(dropJump, "Mighty Blow"));
	}

	// rust: no_mighty_blow_no_injury_modifier
	@Test
	public void noMightyBlowNoInjuryModifier() {
		GameFixture.installScriptedDice(gameState, 6, 6, 1, 1);
		InjuryTypeDropJump dropJump = new InjuryTypeDropJump();
		handleInjury(dropJump, attacker());
		assertTrue(dropJump.injuryContext().isArmorBroken());
		assertFalse(hasInjuryModifier(dropJump, "Mighty Blow"));
	}
}
