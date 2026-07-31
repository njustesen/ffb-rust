package com.fumbbl.ffb.server.mechanic.bb2016;

import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.TurnMode;
import com.fumbbl.ffb.bb2016.SeriousInjury;
import com.fumbbl.ffb.injury.context.InjuryContext;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import java.util.Optional;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/mechanic/bb2016/roll_mechanic.rs tests.
 */
public class RollMechanicTest {

	private GameState gameState;
	private Game game;
	private RollMechanic mechanic;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3);
		game = gameState.getGame();
		mechanic = new RollMechanic();
	}

	// rust: roll_casualty_produces_d6_and_d8
	@Test
	public void rollCasualtyProducesD6AndD8() {
		for (int i = 0; i < 20; i++) {
			int[] roll = mechanic.rollCasualty(gameState.getDiceRoller());
			assertTrue(roll[0] >= 1 && roll[0] <= 6, "d6 in range, got " + roll[0]);
			assertTrue(roll[1] >= 1 && roll[1] <= 8, "d8 in range, got " + roll[1]);
		}
	}

	// rust: multi_block_modifiers_bb2016
	@Test
	public void multiBlockModifiersBb2016() {
		assertEquals(0, mechanic.multiBlockAttackerModifier());
		assertEquals(2, mechanic.multiBlockDefenderModifier());
	}

	// rust: minimum_rolls_bb2016
	@Test
	public void minimumRollsBb2016() {
		assertEquals(4, mechanic.minimumLonerRoll(game.getPlayerById("home1")));
		assertEquals(4, mechanic.minimumProRoll());
	}

	// rust: allows_re_roll_bb2016_modes
	@Test
	public void allowsReRollBb2016Modes() {
		for (TurnMode mode : new TurnMode[]{TurnMode.KICKOFF, TurnMode.PASS_BLOCK, TurnMode.DUMP_OFF}) {
			assertFalse(mechanic.allowsTeamReRoll(mode), mode + " should be prohibited");
		}
		for (TurnMode mode : new TurnMode[]{TurnMode.REGULAR, TurnMode.BLITZ, TurnMode.QUICK_SNAP}) {
			assertTrue(mechanic.allowsTeamReRoll(mode), mode + " should be allowed");
		}
	}

	// rust: no_additional_reroll_property_bb2016
	@Test
	public void noAdditionalRerollPropertyBb2016() {
		assertEquals(Optional.empty(), mechanic.findAdditionalReRollProperty(game.getTurnDataHome()));
	}

	// rust: mascot_unavailable_bb2016
	@Test
	public void mascotUnavailableBb2016() {
		assertFalse(mechanic.isMascotAvailable(gameState, game.getPlayerById("home1")));
	}

	// rust: casualty_badly_hurt_bb2016
	@Test
	public void casualtyBadlyHurtBb2016() {
		InjuryContext context = new InjuryContext();
		for (int die1 : new int[]{1, 2, 3}) {
			context.setCasualtyRoll(new int[]{die1, 1});
			PlayerState result = mechanic.interpretCasualtyRollAndAddModifiers(
				game, context, game.getPlayerById("home1"), false);
			assertEquals(PlayerState.BADLY_HURT, result.getBase(), "die1=" + die1);
		}
	}

	// rust: casualty_serious_injury_bb2016
	@Test
	public void casualtySeriousInjuryBb2016() {
		InjuryContext context = new InjuryContext();
		for (int die1 : new int[]{4, 5}) {
			context.setCasualtyRoll(new int[]{die1, 1});
			PlayerState result = mechanic.interpretCasualtyRollAndAddModifiers(
				game, context, game.getPlayerById("home1"), false);
			assertEquals(PlayerState.SERIOUS_INJURY, result.getBase(), "die1=" + die1);
		}
	}

	// rust: casualty_rip_bb2016
	@Test
	public void casualtyRipBb2016() {
		InjuryContext context = new InjuryContext();
		context.setCasualtyRoll(new int[]{6, 1});
		PlayerState result = mechanic.interpretCasualtyRollAndAddModifiers(
			game, context, game.getPlayerById("home1"), false);
		assertEquals(PlayerState.RIP, result.getBase());
	}

	// rust: casualty_decay_roll_bb2016
	@Test
	public void casualtyDecayRollBb2016() {
		InjuryContext context = new InjuryContext();
		context.setCasualtyRollDecay(new int[]{6, 3});
		PlayerState result = mechanic.interpretCasualtyRollAndAddModifiers(
			game, context, game.getPlayerById("home1"), true);
		assertEquals(PlayerState.RIP, result.getBase());
	}

	// rust: si_roll_serious_injury_d6_4_d8_1
	@Test
	public void siRollSeriousInjuryD64D81() {
		InjuryContext context = new InjuryContext();
		context.setCasualtyRoll(new int[]{4, 1});
		assertEquals(SeriousInjury.BROKEN_RIBS, mechanic.interpretSeriousInjuryRoll(game, context));
	}

	// rust: si_roll_serious_injury_d6_5_d8_2
	@Test
	public void siRollSeriousInjuryD65D82() {
		InjuryContext context = new InjuryContext();
		context.setCasualtyRoll(new int[]{5, 2});
		assertEquals(SeriousInjury.SMASHED_KNEE, mechanic.interpretSeriousInjuryRoll(game, context));
	}

	// rust: si_roll_no_si_for_die1_6
	@Test
	public void siRollNoSiForDie16() {
		InjuryContext context = new InjuryContext();
		context.setCasualtyRoll(new int[]{6, 1});
		assertNull(mechanic.interpretSeriousInjuryRoll(game, context));
	}

	// rust: injury_roll_forced_bb2016
	@Test
	public void injuryRollForcedBb2016() {
		InjuryContext context = new InjuryContext();
		context.setInjury(new PlayerState(PlayerState.STUNNED));
		assertEquals(PlayerState.STUNNED, mechanic.interpretInjuryRoll(game, context).getBase());
	}

	// rust: injury_roll_total_2_stunned_bb2016
	@Test
	public void injuryRollTotal2StunnedBb2016() {
		InjuryContext context = new InjuryContext();
		context.setInjuryRoll(new int[]{1, 1});
		assertEquals(PlayerState.STUNNED, mechanic.interpretInjuryRoll(game, context).getBase());
	}

	// rust: injury_roll_total_10_is_casualty_bb2016
	@Test
	public void injuryRollTotal10IsCasualtyBb2016() {
		InjuryContext context = new InjuryContext();
		context.setInjuryRoll(new int[]{5, 5});
		assertNull(mechanic.interpretInjuryRoll(game, context));
	}
}
