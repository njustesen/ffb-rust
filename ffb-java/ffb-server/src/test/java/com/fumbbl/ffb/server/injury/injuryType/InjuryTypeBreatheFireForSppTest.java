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
 * BreatheFireForSpp is vomit-like and caused-by-opponent; armour save leaves the player prone, a break
 * rolls injury (with niggling modifiers in bb2016). SendToBoxReason is BREATHE_FIRE.
 */
public class InjuryTypeBreatheFireForSppTest {

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

	private void handleInjury(InjuryTypeBreatheFireForSpp breatheFireForSpp) {
		breatheFireForSpp.injuryContext().setDefenderId("away1");
		breatheFireForSpp.handleInjury(step, game, gameState, gameState.getDiceRoller(), null, defender(),
			new FieldCoordinate(5, 5), null, null, ApothecaryMode.DEFENDER);
	}

	private boolean hasInjuryModifier(InjuryTypeBreatheFireForSpp breatheFireForSpp, String namePart) {
		return Arrays.stream(breatheFireForSpp.injuryContext().getInjuryModifiers())
			.anyMatch(m -> m.getName() != null && m.getName().contains(namePart));
	}

	// rust: armor_save_results_in_prone
	@Test
	public void armorSaveResultsInProne() {
		GameFixture.installScriptedDice(gameState, 1, 1);
		InjuryTypeBreatheFireForSpp breatheFireForSpp = new InjuryTypeBreatheFireForSpp();
		handleInjury(breatheFireForSpp);
		assertFalse(breatheFireForSpp.injuryContext().isArmorBroken());
		assertNotNull(breatheFireForSpp.injuryContext().getInjury());
		assertEquals(PlayerState.PRONE, breatheFireForSpp.injuryContext().getInjury().getBase());
	}

	// rust: armor_break_results_in_injury_roll
	@Test
	public void armorBreakResultsInInjuryRoll() {
		GameFixture.installScriptedDice(gameState, 6, 6, 1, 1);
		InjuryTypeBreatheFireForSpp breatheFireForSpp = new InjuryTypeBreatheFireForSpp();
		handleInjury(breatheFireForSpp);
		assertTrue(breatheFireForSpp.injuryContext().isArmorBroken());
		assertNotEquals(PlayerState.PRONE, breatheFireForSpp.injuryContext().getInjury().getBase());
	}

	// rust: turnover_default_true
	@Test
	public void turnoverDefaultTrue() {
		assertTrue(new InjuryTypeBreatheFireForSpp().injuryType().fallingDownCausesTurnover());
	}

	// BreatheFireForSpp.isWorthSpps() is true (the SPP-earning variant)
	@Test
	public void isWorthSpps() {
		assertTrue(new InjuryTypeBreatheFireForSpp().injuryType().isWorthSpps());
	}

	// rust: send_to_box_reason_is_breathe_fire
	@Test
	public void sendToBoxReasonIsBreatheFireForSpp() {
		assertEquals(SendToBoxReason.BREATHE_FIRE, new InjuryTypeBreatheFireForSpp().sendToBoxReason());
	}

	// rust: context_stores_defender_and_coordinate (Java stores only the defender id here — the
	// defender coordinate is populated by the CALLER, not handleInjury, a documented
	// context-storage divergence; the Rust ctx pre-stores the coordinate)
	@Test
	public void contextStoresDefender() {
		GameFixture.installScriptedDice(gameState, 1, 1);
		InjuryTypeBreatheFireForSpp breatheFireForSpp = new InjuryTypeBreatheFireForSpp();
		handleInjury(breatheFireForSpp);
		assertEquals("away1", breatheFireForSpp.injuryContext().getDefenderId());
	}

	// rust: niggling_injury_adds_injury_modifier
	@Test
	public void nigglingInjuryAddsInjuryModifier() {
		init(GameFixture.createGameState(3, RulesCollection.Rules.BB2016));
		defender().addLastingInjury(com.fumbbl.ffb.bb2016.SeriousInjury.SMASHED_KNEE);
		GameFixture.installScriptedDice(gameState, 6, 6, 1, 1);
		InjuryTypeBreatheFireForSpp breatheFireForSpp = new InjuryTypeBreatheFireForSpp();
		handleInjury(breatheFireForSpp);
		assertTrue(hasInjuryModifier(breatheFireForSpp, "Niggling"));
	}

	// rust: no_niggling_injury_no_injury_modifier
	@Test
	public void noNigglingInjuryNoInjuryModifier() {
		init(GameFixture.createGameState(3, RulesCollection.Rules.BB2016));
		GameFixture.installScriptedDice(gameState, 6, 6, 1, 1);
		InjuryTypeBreatheFireForSpp breatheFireForSpp = new InjuryTypeBreatheFireForSpp();
		handleInjury(breatheFireForSpp);
		assertFalse(hasInjuryModifier(breatheFireForSpp, "Niggling"));
	}
}
