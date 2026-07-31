package com.fumbbl.ffb.server.mechanic.bb2025;

import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.ReRollProperty;
import com.fumbbl.ffb.TurnMode;
import com.fumbbl.ffb.bb2025.SeriousInjury;
import com.fumbbl.ffb.injury.context.InjuryContext;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.RosterPlayer;
import com.fumbbl.ffb.model.TurnData;
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
 * Mirror of ffb-rust crates/ffb-engine/src/mechanic/bb2025/roll_mechanic.rs tests.
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

	// rust: roll_casualty_range
	@Test
	public void rollCasualtyRange() {
		for (int i = 0; i < 20; i++) {
			int[] roll = mechanic.rollCasualty(gameState.getDiceRoller());
			assertTrue(roll[0] >= 1 && roll[0] <= 16, "d16 in range, got " + roll[0]);
			assertTrue(roll[1] >= 1 && roll[1] <= 6, "d6 in range, got " + roll[1]);
		}
	}

	// rust: multi_block_modifiers
	@Test
	public void multiBlockModifiers() {
		assertEquals(-2, mechanic.multiBlockAttackerModifier());
		assertEquals(0, mechanic.multiBlockDefenderModifier());
	}

	// rust: minimum_pro_roll_is_3
	@Test
	public void minimumProRollIs3() {
		assertEquals(3, mechanic.minimumProRoll());
	}

	// rust: allows_re_roll_bb2025_modes
	@Test
	public void allowsReRollBb2025Modes() {
		for (TurnMode mode : new TurnMode[]{TurnMode.KICKOFF, TurnMode.PASS_BLOCK, TurnMode.DUMP_OFF,
			TurnMode.QUICK_SNAP, TurnMode.BETWEEN_TURNS}) {
			assertFalse(mechanic.allowsTeamReRoll(mode), mode + " should be prohibited");
		}
		for (TurnMode mode : new TurnMode[]{TurnMode.REGULAR, TurnMode.BLITZ}) {
			assertTrue(mechanic.allowsTeamReRoll(mode), mode + " should be allowed");
		}
	}

	// rust: find_additional_reroll_brilliant_coaching
	@Test
	public void findAdditionalRerollBrilliantCoaching() {
		TurnData turnData = game.getTurnDataHome();
		turnData.setReRollsBrilliantCoachingOneDrive(1);
		assertEquals(Optional.of(ReRollProperty.BRILLIANT_COACHING),
			mechanic.findAdditionalReRollProperty(turnData));
	}

	// rust: find_additional_reroll_pump_up_crowd
	@Test
	public void findAdditionalRerollPumpUpCrowd() {
		TurnData turnData = game.getTurnDataHome();
		turnData.setReRollsPumpUpTheCrowdOneDrive(1);
		assertEquals(Optional.of(ReRollProperty.PUMP_UP_THE_CROWD),
			mechanic.findAdditionalReRollProperty(turnData));
	}

	// rust: find_additional_reroll_show_star
	@Test
	public void findAdditionalRerollShowStar() {
		TurnData turnData = game.getTurnDataHome();
		turnData.setReRollShowStarOneDrive(1);
		assertEquals(Optional.of(ReRollProperty.SHOW_STAR),
			mechanic.findAdditionalReRollProperty(turnData));
	}

	// rust: find_additional_reroll_none
	@Test
	public void findAdditionalRerollNone() {
		assertEquals(Optional.empty(), mechanic.findAdditionalReRollProperty(game.getTurnDataHome()));
	}

	// rust: interpret_casualty_badly_hurt_below_9
	@Test
	public void interpretCasualtyBadlyHurtBelow9() {
		InjuryContext context = new InjuryContext();
		context.setCasualtyRoll(new int[]{8, 1});
		PlayerState result = mechanic.interpretCasualtyRollAndAddModifiers(
			game, context, game.getPlayerById("home1"), false);
		assertEquals(PlayerState.BADLY_HURT, result.getBase());
	}

	// rust: interpret_casualty_serious_injury_at_9
	@Test
	public void interpretCasualtySeriousInjuryAt9() {
		InjuryContext context = new InjuryContext();
		context.setCasualtyRoll(new int[]{9, 1});
		PlayerState result = mechanic.interpretCasualtyRollAndAddModifiers(
			game, context, game.getPlayerById("home1"), false);
		assertEquals(PlayerState.SERIOUS_INJURY, result.getBase());
	}

	// rust: interpret_casualty_rip_at_15
	@Test
	public void interpretCasualtyRipAt15() {
		InjuryContext context = new InjuryContext();
		context.setCasualtyRoll(new int[]{15, 2});
		PlayerState result = mechanic.interpretCasualtyRollAndAddModifiers(
			game, context, game.getPlayerById("home1"), false);
		assertEquals(PlayerState.RIP, result.getBase());
	}

	// rust: interpret_si_roll_seriously_hurt
	@Test
	public void interpretSiRollSeriouslyHurt() {
		InjuryContext context = new InjuryContext();
		context.setCasualtyRoll(new int[]{9, 1});
		assertEquals(SeriousInjury.SERIOUSLY_HURT, mechanic.interpretSeriousInjuryRoll(game, context));
		context.setCasualtyRoll(new int[]{10, 6});
		assertEquals(SeriousInjury.SERIOUSLY_HURT, mechanic.interpretSeriousInjuryRoll(game, context));
	}

	// rust: interpret_si_roll_ni
	@Test
	public void interpretSiRollNi() {
		InjuryContext context = new InjuryContext();
		context.setCasualtyRoll(new int[]{11, 1});
		assertEquals(SeriousInjury.SERIOUS_INJURY, mechanic.interpretSeriousInjuryRoll(game, context));
		context.setCasualtyRoll(new int[]{12, 3});
		assertEquals(SeriousInjury.SERIOUS_INJURY, mechanic.interpretSeriousInjuryRoll(game, context));
	}

	// rust: interpret_si_roll_detail_table_defender_at_threshold_falls_back
	@Test
	public void interpretSiRollDetailTableDefenderAtThresholdFallsBack() {
		((RosterPlayer) game.getPlayerById("away1")).setArmour(3);
		InjuryContext context = new InjuryContext();
		context.setDefenderId("away1");
		context.setCasualtyRoll(new int[]{13, 1}); // d6=1 -> HeadInjury (AV), AV already at threshold 3
		assertEquals(SeriousInjury.SERIOUSLY_HURT, mechanic.interpretSeriousInjuryRoll(game, context));
	}

	// rust: interpret_injury_roll_forced_returns_preset
	@Test
	public void interpretInjuryRollForcedReturnsPreset() {
		InjuryContext context = new InjuryContext();
		context.setInjury(new PlayerState(PlayerState.STUNNED));
		PlayerState result = mechanic.interpretInjuryRoll(game, context);
		assertEquals(PlayerState.STUNNED, result.getBase());
	}

	// rust: interpret_injury_roll_total_2_is_stunned
	@Test
	public void interpretInjuryRollTotal2IsStunned() {
		InjuryContext context = new InjuryContext();
		context.setInjuryRoll(new int[]{1, 1});
		PlayerState result = mechanic.interpretInjuryRoll(game, context);
		assertEquals(PlayerState.STUNNED, result.getBase());
	}

	// rust: interpret_injury_roll_total_10_is_casualty
	@Test
	public void interpretInjuryRollTotal10IsCasualty() {
		InjuryContext context = new InjuryContext();
		context.setInjuryRoll(new int[]{5, 5});
		assertNull(mechanic.interpretInjuryRoll(game, context));
	}
}
