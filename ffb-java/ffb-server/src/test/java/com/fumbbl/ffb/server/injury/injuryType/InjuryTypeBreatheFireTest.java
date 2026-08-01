package com.fumbbl.ffb.server.injury.injuryType;

import com.fumbbl.ffb.ApothecaryMode;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.RulesCollection;
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
 * Mirror of ffb-rust crates/ffb-engine/src/injury/injuryType/injury_type_breathe_fire.rs tests.
 * BreatheFire is vomit-like and caused-by-opponent; armour save leaves the player prone, a break
 * rolls injury (with niggling modifiers in bb2016). SendToBoxReason is BREATHE_FIRE.
 */
public class InjuryTypeBreatheFireTest {

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
		GameFixture.placePlayer(gameState, "home1", 2, 2);
		GameFixture.placePlayer(gameState, "away1", 5, 5);
	}

	private RosterPlayer defender() {
		return (RosterPlayer) game.getPlayerById("away1");
	}

	private void handleInjury(InjuryTypeBreatheFire breatheFire) {
		breatheFire.injuryContext().setDefenderId("away1");
		breatheFire.handleInjury(step, game, gameState, gameState.getDiceRoller(), null, defender(),
			new FieldCoordinate(5, 5), null, null, ApothecaryMode.DEFENDER);
	}

	private boolean hasInjuryModifier(InjuryTypeBreatheFire breatheFire, String namePart) {
		return Arrays.stream(breatheFire.injuryContext().getInjuryModifiers())
			.anyMatch(m -> m.getName() != null && m.getName().contains(namePart));
	}

	// rust: armor_save_results_in_prone
	@Test
	public void armorSaveResultsInProne() {
		GameFixture.installScriptedDice(gameState, 1, 1);
		InjuryTypeBreatheFire breatheFire = new InjuryTypeBreatheFire();
		handleInjury(breatheFire);
		assertFalse(breatheFire.injuryContext().isArmorBroken());
		assertNotNull(breatheFire.injuryContext().getInjury());
		assertEquals(PlayerState.PRONE, breatheFire.injuryContext().getInjury().getBase());
	}

	// rust: armor_break_results_in_injury_roll
	@Test
	public void armorBreakResultsInInjuryRoll() {
		GameFixture.installScriptedDice(gameState, 6, 6, 1, 1);
		InjuryTypeBreatheFire breatheFire = new InjuryTypeBreatheFire();
		handleInjury(breatheFire);
		assertTrue(breatheFire.injuryContext().isArmorBroken());
		assertNotEquals(PlayerState.PRONE, breatheFire.injuryContext().getInjury().getBase());
	}

	// rust: turnover_default_true
	@Test
	public void turnoverDefaultTrue() {
		assertTrue(new InjuryTypeBreatheFire().injuryType().fallingDownCausesTurnover());
	}

	// rust: send_to_box_reason_is_breathe_fire
	@Test
	public void sendToBoxReasonIsBreatheFire() {
		assertEquals(SendToBoxReason.BREATHE_FIRE, new InjuryTypeBreatheFire().sendToBoxReason());
	}

	// rust: context_stores_defender_and_coordinate (Java stores only the defender id here — the
	// defender coordinate is populated by the CALLER, not handleInjury, a documented
	// context-storage divergence; the Rust ctx pre-stores the coordinate)
	@Test
	public void contextStoresDefender() {
		GameFixture.installScriptedDice(gameState, 1, 1);
		InjuryTypeBreatheFire breatheFire = new InjuryTypeBreatheFire();
		handleInjury(breatheFire);
		assertEquals("away1", breatheFire.injuryContext().getDefenderId());
	}

	// rust: niggling_injury_adds_injury_modifier
	@Test
	public void nigglingInjuryAddsInjuryModifier() {
		init(GameFixture.createGameState(3, RulesCollection.Rules.BB2016));
		defender().addLastingInjury(com.fumbbl.ffb.bb2016.SeriousInjury.SMASHED_KNEE);
		GameFixture.installScriptedDice(gameState, 6, 6, 1, 1);
		InjuryTypeBreatheFire breatheFire = new InjuryTypeBreatheFire();
		handleInjury(breatheFire);
		assertTrue(hasInjuryModifier(breatheFire, "Niggling"));
	}

	// rust: no_niggling_injury_no_injury_modifier
	@Test
	public void noNigglingInjuryNoInjuryModifier() {
		init(GameFixture.createGameState(3, RulesCollection.Rules.BB2016));
		GameFixture.installScriptedDice(gameState, 6, 6, 1, 1);
		InjuryTypeBreatheFire breatheFire = new InjuryTypeBreatheFire();
		handleInjury(breatheFire);
		assertFalse(hasInjuryModifier(breatheFire, "Niggling"));
	}
}
