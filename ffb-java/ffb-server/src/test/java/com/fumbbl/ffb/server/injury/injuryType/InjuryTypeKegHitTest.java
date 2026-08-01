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
 * Mirror of ffb-rust crates/ffb-engine/src/injury/injuryType/injury_type_keg_hit.rs tests.
 */
public class InjuryTypeKegHitTest {

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

	private void handleInjury(InjuryTypeKegHit injuryType) {
		injuryType.injuryContext().setAttackerId("home1");
		injuryType.injuryContext().setDefenderId("away1");
		injuryType.handleInjury(step, game, gameState, gameState.getDiceRoller(), attacker(), defender(),
			new FieldCoordinate(5, 5), null, null, ApothecaryMode.DEFENDER);
	}

	private boolean hasInjuryModifier(InjuryTypeKegHit injuryType, String namePart) {
		return Arrays.stream(injuryType.injuryContext().getInjuryModifiers())
			.anyMatch(m -> m.getName() != null && m.getName().contains(namePart));
	}

	// rust: armor_save_results_in_prone
	@Test
	public void armorSaveResultsInProne() {
		defender().setArmour(13);
		GameFixture.installScriptedDice(gameState, 1, 1);
		InjuryTypeKegHit kegHit = new InjuryTypeKegHit();
		handleInjury(kegHit);
		assertEquals(PlayerState.PRONE, kegHit.injuryContext().getInjury().getBase());
	}

	// rust: armor_break_results_in_injury_roll
	@Test
	public void armorBreakResultsInInjuryRoll() {
		defender().setArmour(2);
		GameFixture.installScriptedDice(gameState, 6, 6, 1, 1);
		InjuryTypeKegHit kegHit = new InjuryTypeKegHit();
		handleInjury(kegHit);
		assertTrue(kegHit.injuryContext().isArmorBroken());
		assertNotEquals(PlayerState.PRONE, kegHit.injuryContext().getInjury().getBase());
	}

	// rust: niggling_injury_modifier_applied_when_armor_breaks
	@Test
	public void nigglingInjuryModifierAppliedWhenArmorBreaks() {
		init(GameFixture.createGameState(3, com.fumbbl.ffb.RulesCollection.Rules.BB2016));
		defender().setArmour(2);
		defender().addLastingInjury(com.fumbbl.ffb.bb2016.SeriousInjury.SMASHED_KNEE);
		GameFixture.installScriptedDice(gameState, 6, 6, 1, 1);
		InjuryTypeKegHit kegHit = new InjuryTypeKegHit();
		handleInjury(kegHit);
		assertTrue(hasInjuryModifier(kegHit, "Niggling"));
	}

	// rust: no_niggling_injury_no_modifier
	@Test
	public void noNigglingInjuryNoModifier() {
		init(GameFixture.createGameState(3, com.fumbbl.ffb.RulesCollection.Rules.BB2016));
		defender().setArmour(2);
		GameFixture.installScriptedDice(gameState, 6, 6, 1, 1);
		InjuryTypeKegHit kegHit = new InjuryTypeKegHit();
		handleInjury(kegHit);
		assertFalse(hasInjuryModifier(kegHit, "Niggling"));
	}

	// rust: send_to_box_reason_is_thrown_keg
	@Test
	public void sendToBoxReasonIsThrownKeg() {
		assertEquals(SendToBoxReason.THROWN_KEG, new InjuryTypeKegHit().sendToBoxReason());
	}
}
