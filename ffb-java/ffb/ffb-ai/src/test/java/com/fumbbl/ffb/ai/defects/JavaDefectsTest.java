package com.fumbbl.ffb.ai.defects;

import com.fumbbl.ffb.ApothecaryMode;
import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.injury.context.InjuryContext;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.net.commands.ClientCommandEndTurn;
import com.fumbbl.ffb.server.InjuryResult;
import com.fumbbl.ffb.server.injury.injuryType.InjuryTypeStab;
import com.fumbbl.ffb.server.step.StepId;
import com.fumbbl.ffb.server.step.bb2025.StepEndFuriousOutburst;
import com.fumbbl.ffb.server.step.generator.FuriousOutburst;
import com.fumbbl.ffb.server.step.generator.SequenceGenerator;
import com.fumbbl.ffb.server.util.UtilServerInjury;
import org.junit.jupiter.api.Test;

import java.util.Arrays;

import static org.junit.jupiter.api.Assertions.*;

/**
 * One test per entry in ffb-rust/docs/JAVA_DEFECTS.md: each drives STOCK Java (ffb-server is not
 * modified) into the defect and asserts what Java actually does. If a Java update fixes a defect,
 * its test fails, and the entry (and the Rust port of the defect) must be revisited.
 */
class JavaDefectsTest {

	// ── JD-001 ──────────────────────────────────────────────────────────────────────────────────

	/**
	 * JD-001: a bb2025 punt goes out of bounds, the coach re-rolls the distance and it lands on the
	 * pitch. outOfBounds is never cleared, so the step publishes a THROW-IN from an interior square,
	 * and ThrowInMechanic has no direction for an interior square.
	 */
	@Test
	void jd001_rerolledPuntDistanceKeepsStaleOutOfBoundsAndThrowInThrows() throws Exception {
		DefectFixture f = DefectFixture.create("BB2025", "human");
		Player<?> punter = f.home(1);
		f.place(punter, 5, 5, PlayerState.STANDING);
		f.act(punter, PlayerAction.PUNT);
		f.game.getTurnDataHome().setReRolls(1);

		DefectFixture.Recorder next = f.recorder(StepId.CATCH_SCATTER_THROW_IN);
		com.fumbbl.ffb.server.step.bb2025.punt.StepPuntDistance punt =
			new com.fumbbl.ffb.server.step.bb2025.punt.StepPuntDistance(f.gameState);
		punt.setParameter(new com.fumbbl.ffb.server.step.StepParameter(
			com.fumbbl.ffb.server.step.StepParameterKey.COORDINATE_FROM, new com.fumbbl.ffb.FieldCoordinate(5, 5)));
		punt.setParameter(new com.fumbbl.ffb.server.step.StepParameter(
			com.fumbbl.ffb.server.step.StepParameterKey.DIRECTION, com.fumbbl.ffb.Direction.WEST));
		f.pushStep(punt);

		f.dice(6, 2);   // distance 6 west from x=5: off the pitch; re-rolled 2: x=3, on the pitch
		f.startNextStep();
		assertTrue(f.game.getFieldModel().isOutOfBounds(), "first roll: out of bounds");
		assertSame(punt, f.current(), "the team re-roll is offered");

		f.send(new com.fumbbl.ffb.net.commands.ClientCommandUseReRoll(
			com.fumbbl.ffb.ReRolledActions.PUNT_DISTANCE, com.fumbbl.ffb.ReRollSources.TEAM_RE_ROLL), true);
		com.fumbbl.ffb.FieldCoordinate ball = f.game.getFieldModel().getBallCoordinate();
		assertEquals(new com.fumbbl.ffb.FieldCoordinate(3, 5), ball, "re-rolled: the ball is on an interior square");
		assertTrue(next.reached);
		assertTrue(f.game.getFieldModel().isOutOfBounds(), "DEFECT: outOfBounds is still set from the first roll");
		assertEquals(com.fumbbl.ffb.CatchScatterThrowInMode.THROW_IN,
			next.published.get(com.fumbbl.ffb.server.step.StepParameterKey.CATCH_SCATTER_THROW_IN_MODE));
		assertEquals(ball, next.published.get(com.fumbbl.ffb.server.step.StepParameterKey.THROW_IN_COORDINATE),
			"DEFECT: a throw-in is published from the interior square");

		com.fumbbl.ffb.mechanics.ThrowInMechanic throwIn = (com.fumbbl.ffb.mechanics.ThrowInMechanic)
			f.game.getFactory(FactoryType.Factory.MECHANIC).forName(com.fumbbl.ffb.mechanics.Mechanic.Type.THROW_IN.name());
		IllegalStateException crash = assertThrows(IllegalStateException.class,
			() -> throwIn.interpretThrowInDirectionRoll(ball, 3));
		assertEquals("Unable to determine throwInDirection.", crash.getMessage());
	}

	// ── JD-002 ──────────────────────────────────────────────────────────────────────────────────

	/**
	 * JD-002: a foul on a prone Ball & Chain player. handleInjury marks its armour broken WITHOUT an
	 * armour roll, and the referee hook (SneakyGitBehaviour, run for every foul) reads armorRoll[0].
	 */
	@Test
	void jd002_refereeOnABallAndChainFoulThrowsNpe() throws Exception {
		DefectFixture f = DefectFixture.create("BB2025", "goblin");
		Player<?> fouler = f.home(1);
		Player<?> fanatic = f.away(6);
		f.place(fouler, 10, 7, PlayerState.STANDING);
		f.place(fanatic, 11, 7, PlayerState.PRONE);
		f.act(fouler, PlayerAction.FOUL);

		f.dice(3, 4, 3, 4, 3, 4);   // injury dice only: no armour roll is asked for
		InjuryResult foul = UtilServerInjury.handleInjury(new StepEndFuriousOutburst(f.gameState),
			new com.fumbbl.ffb.server.injury.injuryType.InjuryTypeFoul(false), fouler, fanatic,
			f.game.getFieldModel().getPlayerCoordinate(fanatic), null, null, ApothecaryMode.DEFENDER);
		assertTrue(foul.injuryContext().isArmorBroken(), "Ball & Chain: armour broken automatically");
		assertNull(foul.injuryContext().getArmorRoll(), "... with no armour dice");

		com.fumbbl.ffb.server.step.action.foul.StepReferee referee =
			new com.fumbbl.ffb.server.step.action.foul.StepReferee(f.gameState);
		referee.setParameter(new com.fumbbl.ffb.server.step.StepParameter(
			com.fumbbl.ffb.server.step.StepParameterKey.INJURY_RESULT, foul));
		NullPointerException npe = assertThrows(NullPointerException.class, referee::start);
		assertTrue(npe.getMessage().contains("armorRoll"), npe.getMessage());
	}

	// ── JD-004 ──────────────────────────────────────────────────────────────────────────────────

	/**
	 * JD-004: bb2020 StepJump. A failed leap is re-rolled with a team re-roll and succeeds; Diving
	 * Tackle is offered; on the answer executeStep runs again with JUMP still the re-rolled action
	 * and spends the team re-roll a second time (bb2025's StepJump guards this with usingDivingTackle).
	 */
	@Test
	void jd004_bb2020LeapSpendsTheTeamReRollAgainAfterDivingTackleAnswer() throws Exception {
		DefectFixture f = DefectFixture.create("BB2020", "human");
		Player<?> leaper = f.home(1);
		Player<?> tackler = f.away(1);
		f.give(leaper, com.fumbbl.ffb.skill.bb2020.Leap.class);
		f.give(tackler, com.fumbbl.ffb.skill.mixed.DivingTackle.class);
		f.place(leaper, 12, 7, PlayerState.MOVING);      // already on the landing square, as in a move
		f.place(tackler, 9, 7, PlayerState.STANDING);    // next to the take-off square
		f.act(leaper, PlayerAction.MOVE);
		f.game.getActingPlayer().setJumping(true);
		f.game.getTurnDataHome().setReRolls(1);

		DefectFixture.Recorder failed = f.recorder(StepId.FALL_DOWN);
		failed.setLabel("failed");
		DefectFixture.Recorder next = f.recorder(StepId.MOVE);
		com.fumbbl.ffb.server.step.bb2020.move.StepJump jump = new com.fumbbl.ffb.server.step.bb2020.move.StepJump(f.gameState);
		com.fumbbl.ffb.server.step.StepParameterSet params = new com.fumbbl.ffb.server.step.StepParameterSet();
		params.add(new com.fumbbl.ffb.server.step.StepParameter(
			com.fumbbl.ffb.server.step.StepParameterKey.GOTO_LABEL_ON_FAILURE, "failed"));
		params.add(new com.fumbbl.ffb.server.step.StepParameter(
			com.fumbbl.ffb.server.step.StepParameterKey.MOVE_START, new com.fumbbl.ffb.FieldCoordinate(10, 7)));
		jump.init(params);
		f.pushStep(jump);

		f.dice(1, 4, 4, 4);   // leap fails on 1; re-rolled 4 makes it, but not with Diving Tackle's -2
		f.startNextStep();
		assertSame(jump, f.current(), "the team re-roll is offered");
		f.send(new com.fumbbl.ffb.net.commands.ClientCommandUseReRoll(
			com.fumbbl.ffb.ReRolledActions.JUMP, com.fumbbl.ffb.ReRollSources.TEAM_RE_ROLL), true);
		assertFalse(failed.reached, "the re-rolled leap made it");
		assertEquals(0, f.game.getTurnDataHome().getReRolls(), "one team re-roll spent on the re-roll");
		assertSame(jump, f.current(), "the leap made it; Diving Tackle is offered to the away coach");
		assertEquals(com.fumbbl.ffb.dialog.DialogId.PLAYER_CHOICE, f.game.getDialogParameter().getId());

		f.send(new com.fumbbl.ffb.net.commands.ClientCommandPlayerChoice(
			com.fumbbl.ffb.PlayerChoiceMode.DIVING_TACKLE, new Player<?>[]{tackler}), false);
		System.out.println("JD004 dice: " + f.rolled() + " reRolls=" + f.game.getTurnDataHome().getReRolls()
			+ " reached=" + next.reached);
		assertEquals(-1, f.game.getTurnDataHome().getReRolls(),
			"DEFECT: the Diving Tackle answer spent the team re-roll again (bank 1 -> -1)");
	}

	// ── JD-005 ──────────────────────────────────────────────────────────────────────────────────

	/**
	 * JD-005: bb2025 StepMoveDodge. A dodge fails, is team-re-rolled and fails again; two Arm Bar
	 * players qualify, so the opposing coach chooses one. The choice re-runs executeStep with DODGE
	 * still the re-rolled action: the team re-roll is spent again and a third dodge die is rolled.
	 */
	@Test
	void jd005_bb2025ArmBarChoiceSpendsTheReRollAgainAndRollsAThirdDodge() throws Exception {
		DefectFixture f = DefectFixture.create("BB2025", "human");
		Player<?> dodger = f.home(1);
		Player<?> armBar1 = f.away(1);
		Player<?> armBar2 = f.away(2);
		f.give(armBar1, com.fumbbl.ffb.skill.mixed.ArmBar.class);
		f.give(armBar2, com.fumbbl.ffb.skill.mixed.ArmBar.class);
		f.place(dodger, 12, 7, PlayerState.MOVING);     // has stepped from (11,7) to (12,7)
		f.place(armBar1, 10, 7, PlayerState.STANDING);  // both mark the square left, neither the target
		f.place(armBar2, 10, 8, PlayerState.STANDING);
		f.act(dodger, PlayerAction.MOVE);
		f.game.getActingPlayer().setDodging(true);
		f.game.getTurnDataHome().setReRolls(2);

		DefectFixture.Recorder failed = f.recorder(StepId.FALL_DOWN);
		failed.setLabel("failed");
		DefectFixture.Recorder next = f.recorder(StepId.MOVE);
		com.fumbbl.ffb.server.step.bb2025.move.StepMoveDodge dodge = new com.fumbbl.ffb.server.step.bb2025.move.StepMoveDodge(f.gameState);
		com.fumbbl.ffb.server.step.StepParameterSet params = new com.fumbbl.ffb.server.step.StepParameterSet();
		params.add(new com.fumbbl.ffb.server.step.StepParameter(
			com.fumbbl.ffb.server.step.StepParameterKey.GOTO_LABEL_ON_FAILURE, "failed"));
		dodge.init(params);
		dodge.setParameter(new com.fumbbl.ffb.server.step.StepParameter(
			com.fumbbl.ffb.server.step.StepParameterKey.COORDINATE_FROM, new com.fumbbl.ffb.FieldCoordinate(11, 7)));
		dodge.setParameter(new com.fumbbl.ffb.server.step.StepParameter(
			com.fumbbl.ffb.server.step.StepParameterKey.COORDINATE_TO, new com.fumbbl.ffb.FieldCoordinate(12, 7)));
		f.pushStep(dodge);

		f.dice(1, 1, 6, 6);
		f.startNextStep();
		assertSame(dodge, f.current(), "the team re-roll is offered");
		f.send(new com.fumbbl.ffb.net.commands.ClientCommandUseReRoll(
			com.fumbbl.ffb.ReRolledActions.DODGE, com.fumbbl.ffb.ReRollSources.TEAM_RE_ROLL), true);
		assertEquals(1, f.game.getTurnDataHome().getReRolls(), "one team re-roll spent");
		assertEquals(Arrays.asList("d6=1", "d6=1"), f.rolled(), "dodge and its re-roll both failed");
		assertSame(dodge, f.current(), "two Arm Bar players: the away coach is asked to choose");
		assertEquals(com.fumbbl.ffb.dialog.DialogId.PLAYER_CHOICE, f.game.getDialogParameter().getId());

		f.send(new com.fumbbl.ffb.net.commands.ClientCommandPlayerChoice(
			com.fumbbl.ffb.PlayerChoiceMode.ARM_BAR, new Player<?>[]{armBar1}), false);
		System.out.println("JD005 dice: " + f.rolled() + " reRolls=" + f.game.getTurnDataHome().getReRolls()
			+ " failed=" + failed.reached + " next=" + next.reached);
		assertEquals(0, f.game.getTurnDataHome().getReRolls(),
			"DEFECT: answering the Arm Bar choice spent a second team re-roll");
		assertEquals(Arrays.asList("d6=1", "d6=1", "d6=6"), f.rolled(),
			"DEFECT: ... and rolled a third dodge die, which here SUCCEEDS");
		assertTrue(next.reached && !failed.reached, "the twice-failed dodger is still standing");
	}

	// ── JD-009 (withdrawn) ─────────────────────────────────────────────────────────────────────

	/**
	 * Withdrawn JD-009: bb2025 DivingTackleBehaviour adds the DT modifier to dodgeModifiers instead
	 * of dodgeModifiersWithBT, so strengthModifierCanBeAdded is always false in that branch. But the
	 * branch only runs when the dodger has NO unused Break Tackle, where a "will force the use of
	 * BREAK TACKLE" hint would itself be wrong. Observable result: no Break Tackle hint -- correct.
	 */
	@Test
	void notADefect_divingTackleHintWithBreakTackleSpent() throws Exception {
		DefectFixture f = DefectFixture.create("BB2025", "human");
		Player<?> dodger = f.home(1);
		Player<?> tackler = f.away(1);
		f.give(dodger, com.fumbbl.ffb.skill.bb2025.BreakTackle.class);
		f.give(tackler, com.fumbbl.ffb.skill.mixed.DivingTackle.class);
		f.place(dodger, 12, 7, PlayerState.MOVING);
		f.place(tackler, 10, 7, PlayerState.STANDING);   // marks the square left, not the target
		f.act(dodger, PlayerAction.MOVE);
		f.game.getActingPlayer().markSkillUsed(f.skill(com.fumbbl.ffb.skill.bb2025.BreakTackle.class));

		f.recorder(StepId.MOVE).setLabel("success");
		com.fumbbl.ffb.server.step.action.move.StepDivingTackle dt =
			new com.fumbbl.ffb.server.step.action.move.StepDivingTackle(f.gameState);
		com.fumbbl.ffb.server.step.StepParameterSet params = new com.fumbbl.ffb.server.step.StepParameterSet();
		params.add(new com.fumbbl.ffb.server.step.StepParameter(
			com.fumbbl.ffb.server.step.StepParameterKey.GOTO_LABEL_ON_SUCCESS, "success"));
		dt.init(params);
		dt.setParameter(new com.fumbbl.ffb.server.step.StepParameter(
			com.fumbbl.ffb.server.step.StepParameterKey.COORDINATE_FROM, new com.fumbbl.ffb.FieldCoordinate(11, 7)));
		dt.setParameter(new com.fumbbl.ffb.server.step.StepParameter(
			com.fumbbl.ffb.server.step.StepParameterKey.COORDINATE_TO, new com.fumbbl.ffb.FieldCoordinate(12, 7)));
		dt.setParameter(new com.fumbbl.ffb.server.step.StepParameter(
			com.fumbbl.ffb.server.step.StepParameterKey.DODGE_ROLL, 3));   // makes 3+, not with DT
		f.pushStep(dt);
		f.startNextStep();

		assertSame(dt, f.current(), "Diving Tackle is offered");
		com.fumbbl.ffb.dialog.DialogPlayerChoiceParameter dialog =
			(com.fumbbl.ffb.dialog.DialogPlayerChoiceParameter) f.game.getDialogParameter();
		System.out.println("JD009 descriptions: " + Arrays.toString(dialog.getDescriptions()));
		assertTrue(dialog.getDescriptions() == null || Arrays.stream(dialog.getDescriptions())
				.noneMatch(d -> d.contains("BREAK TACKLE")),
			"no Break Tackle hint for a dodger whose Break Tackle is spent -- the right answer");
	}

	// ── Suspect: StepBlockStatistics NPE after Dump-Off (not a defect) ───────────────────────────

	/**
	 * The 2026-08 NPE in StepBlockStatistics (getPlayerResult(acting player) == null) happened only
	 * while the harness could not answer the Dump-Off pass. Driven as the real client drives it --
	 * use Dump-Off, then throw a Quick Pass (DumpOffLogicModule can only send CLIENT_PASS to a
	 * square in Quick Pass range; it has no way to decline) -- the acting player is never touched,
	 * the block is counted and nothing throws.
	 */
	@Test
	void notADefect_dumpOffThenBlockStatistics() throws Exception {
		DefectFixture f = DefectFixture.create("BB2025", "human");
		Player<?> blocker = f.home(1);
		Player<?> carrier = f.away(1);
		f.give(carrier, com.fumbbl.ffb.skill.common.DumpOff.class);
		f.place(blocker, 10, 7, PlayerState.STANDING);
		f.place(carrier, 11, 7, PlayerState.STANDING);
		f.place(f.away(2), 13, 7, PlayerState.STANDING);
		f.game.getFieldModel().setBallCoordinate(new com.fumbbl.ffb.FieldCoordinate(11, 7));
		f.game.getFieldModel().setBallInPlay(true);
		f.game.getFieldModel().setBallMoving(false);
		f.act(blocker, PlayerAction.BLOCK);
		f.game.setDefenderId(carrier.getId());

		DefectFixture.Recorder after = f.recorder(StepId.DAUNTLESS);
		f.pushStep(new com.fumbbl.ffb.server.step.action.block.StepBlockStatistics(f.gameState));
		f.pushStep(new com.fumbbl.ffb.server.step.action.block.StepDumpOff(f.gameState));
		f.dice(6, 6, 6, 6, 6, 6);

		f.startNextStep();
		assertEquals(StepId.DUMP_OFF, f.current().getId(), "Dump-Off is offered to the ball carrier");
		f.send(new com.fumbbl.ffb.net.commands.ClientCommandUseSkill(
			f.skill(com.fumbbl.ffb.skill.common.DumpOff.class), true, carrier.getId(), null, false), false);
		assertEquals(com.fumbbl.ffb.TurnMode.DUMP_OFF, f.game.getTurnMode());
		assertEquals(StepId.INIT_PASSING, f.current().getId(), "the dump-off pass waits for the carrier's coach");

		f.send(new com.fumbbl.ffb.net.commands.ClientCommandPass(carrier.getId(),
			new com.fumbbl.ffb.FieldCoordinate(13, 7)), false);
		System.out.println("DUMPOFF current=" + f.current().getId() + " turnMode=" + f.game.getTurnMode()
			+ " acting=" + f.game.getActingPlayer().getPlayerId() + " dice=" + f.rolled());
		assertTrue(after.reached, "the sequence ran on through BLOCK_STATISTICS without throwing");
		assertEquals(blocker, f.game.getActingPlayer().getPlayer(), "the acting player is still the blocker");
		assertEquals(1, f.game.getGameResult().getPlayerResult(blocker).getBlocks(), "the block was counted");
	}

	// ── Suspect: bb2020 kicked player landing in the crowd (not a defect) ────────────────────────

	/**
	 * bb2020 StepDispatchScatterPlayer consumes IS_KICKED_PLAYER and does not hand it on, so
	 * INIT_SCATTER_PLAYER's kicked-player crowd arm (InjuryTypeKTMCrowd) never runs and a kicked
	 * player who lands in the crowd takes InjuryTypeCrowdPush. That is the BB2020 rule: Kick
	 * Team-mate follows the Throw Team-mate rules except on a fumble, and a thrown player landing in
	 * the crowd suffers Injury by the Crowd (rules/bb2020/core_rules: "LANDING IN THE CROWD").
	 */
	@Test
	void notADefect_bb2020KickedPlayerIsNotFlaggedForTheScatter() throws Exception {
		DefectFixture f = DefectFixture.create("BB2020", "human");
		Player<?> kicker = f.home(1);
		Player<?> kicked = f.home(2);
		f.place(kicker, 10, 7, PlayerState.STANDING);
		f.place(kicked, 11, 7, PlayerState.STANDING);
		f.act(kicker, PlayerAction.KICK_TEAM_MATE);

		com.fumbbl.ffb.server.step.bb2020.ttm.StepDispatchScatterPlayer dispatch =
			new com.fumbbl.ffb.server.step.bb2020.ttm.StepDispatchScatterPlayer(f.gameState);
		f.pushStep(dispatch);
		// the KTM steps publish these while DISPATCH_SCATTER_PLAYER is on the stack
		f.gameState.getStepStack().publishStepParameter(new com.fumbbl.ffb.server.step.StepParameter(
			com.fumbbl.ffb.server.step.StepParameterKey.IS_KICKED_PLAYER, true));
		dispatch.setParameter(new com.fumbbl.ffb.server.step.StepParameter(
			com.fumbbl.ffb.server.step.StepParameterKey.THROWN_PLAYER_ID, kicked.getId()));
		dispatch.setParameter(new com.fumbbl.ffb.server.step.StepParameter(
			com.fumbbl.ffb.server.step.StepParameterKey.THROWN_PLAYER_STATE, new PlayerState(PlayerState.STANDING)));
		dispatch.setParameter(new com.fumbbl.ffb.server.step.StepParameter(
			com.fumbbl.ffb.server.step.StepParameterKey.PASS_RESULT, com.fumbbl.ffb.mechanics.PassResult.ACCURATE));
		dispatch.setParameter(new com.fumbbl.ffb.server.step.StepParameter(
			com.fumbbl.ffb.server.step.StepParameterKey.OLD_DEFENDER_STATE, new PlayerState(PlayerState.STANDING)));
		f.gameState.getStepStack().pop();
		dispatch.start();

		com.fumbbl.ffb.server.step.IStep top = f.gameState.getStepStack().peek();
		assertEquals(StepId.INIT_SCATTER_PLAYER, top.getId());
		java.lang.reflect.Field flag = top.getClass().getDeclaredField("isKickedPlayer");
		flag.setAccessible(true);
		assertFalse((Boolean) flag.get(top),
			"bb2020: the scatter of a kicked player is not flagged, so a crowd landing is a crowd-push injury");
	}

	// ── Suspect: Animal Savagery catcherId after a move-then-give (not a defect) ─────────────────

	/**
	 * A give declared at the end of a move pushes the Pass sequence with a null target, so
	 * StepAnimalSavagery's catcherId stays null. It cannot matter: Animal Savagery is rolled once per
	 * activation (hasUnusedSkill), and in a move-then-give it was already rolled in the Move
	 * sequence, before any catcher existed. The Pass sequence's step does not roll at all.
	 */
	@Test
	void notADefect_animalSavageryInTheGiveAfterAMoveDoesNotRoll() throws Exception {
		DefectFixture f = DefectFixture.create("BB2020", "human");
		Player<?> savage = f.home(1);
		f.give(savage, com.fumbbl.ffb.skill.mixed.AnimalSavagery.class);
		f.place(savage, 10, 7, PlayerState.MOVING);
		f.place(f.home(2), 11, 7, PlayerState.STANDING);
		f.act(savage, PlayerAction.HAND_OVER_MOVE);
		f.game.getActingPlayer().markSkillUsed(f.skill(com.fumbbl.ffb.skill.mixed.AnimalSavagery.class)); // rolled in the Move sequence

		com.fumbbl.ffb.server.step.mixed.shared.StepAnimalSavagery step =
			new com.fumbbl.ffb.server.step.mixed.shared.StepAnimalSavagery(f.gameState);
		com.fumbbl.ffb.server.step.StepParameterSet params = new com.fumbbl.ffb.server.step.StepParameterSet();
		params.add(new com.fumbbl.ffb.server.step.StepParameter(
			com.fumbbl.ffb.server.step.StepParameterKey.GOTO_LABEL_ON_FAILURE, "endPassing"));
		params.add(new com.fumbbl.ffb.server.step.StepParameter(
			com.fumbbl.ffb.server.step.StepParameterKey.TARGET_COORDINATE, null));   // StepEndMoving's give
		step.init(params);
		f.dice();   // no die may be asked for
		step.start();
		assertEquals(com.fumbbl.ffb.server.step.StepAction.NEXT_STEP, step.getResult().getNextAction());
		assertTrue(f.rolled().isEmpty(), "no Animal Savagery roll in the Pass sequence: " + f.rolled());
	}

	// ── Suspect: bb2016 Diving Tackle marks Break Tackle used on a dodge it trips (not a defect) ──

	/**
	 * bb2016 DivingTackleBehaviour: when Diving Tackle is used, the tail marks Break Tackle used if the
	 * dodge would have made it with Break Tackle but without Diving Tackle's -2. Here AG2/ST4, dodge
	 * roll 3: AG alone needs 4, Break Tackle (ST) needs 2, so Break Tackle WAS what made the roll --
	 * a genuine use. Diving Tackle then trips the dodger, a failed dodge is a turnover, and a
	 * once-per-turn skill marked used at a turnover has no consequence.
	 */
	@Test
	void notADefect_bb2016DivingTackleMarksBreakTackleThatMadeTheRoll() throws Exception {
		DefectFixture f = DefectFixture.create("BB2016", "human");
		Player<?> dodger = f.home(1);
		Player<?> tackler = f.away(1);
		((com.fumbbl.ffb.model.RosterPlayer) dodger).setAgility(2);
		((com.fumbbl.ffb.model.RosterPlayer) dodger).setStrength(4);
		f.give(dodger, com.fumbbl.ffb.skill.bb2016.BreakTackle.class);
		f.give(tackler, com.fumbbl.ffb.skill.bb2016.DivingTackle.class);
		f.place(dodger, 12, 7, PlayerState.MOVING);
		f.place(tackler, 10, 7, PlayerState.STANDING);
		f.act(dodger, PlayerAction.MOVE);

		f.recorder(StepId.MOVE).setLabel("tripped");
		DefectFixture.Recorder dodgeMade = f.recorder(StepId.MOVE_DODGE);
		com.fumbbl.ffb.server.step.action.move.StepDivingTackle dt =
			new com.fumbbl.ffb.server.step.action.move.StepDivingTackle(f.gameState);
		com.fumbbl.ffb.server.step.StepParameterSet params = new com.fumbbl.ffb.server.step.StepParameterSet();
		params.add(new com.fumbbl.ffb.server.step.StepParameter(
			com.fumbbl.ffb.server.step.StepParameterKey.GOTO_LABEL_ON_SUCCESS, "tripped"));
		dt.init(params);
		dt.setParameter(new com.fumbbl.ffb.server.step.StepParameter(
			com.fumbbl.ffb.server.step.StepParameterKey.COORDINATE_FROM, new com.fumbbl.ffb.FieldCoordinate(11, 7)));
		dt.setParameter(new com.fumbbl.ffb.server.step.StepParameter(
			com.fumbbl.ffb.server.step.StepParameterKey.COORDINATE_TO, new com.fumbbl.ffb.FieldCoordinate(12, 7)));
		dt.setParameter(new com.fumbbl.ffb.server.step.StepParameter(
			com.fumbbl.ffb.server.step.StepParameterKey.DODGE_ROLL, 3));
		f.pushStep(dt);
		f.startNextStep();
		assertSame(dt, f.current(), "3 fails with Break Tackle AND Diving Tackle (needs 4): DT is offered");

		f.send(new com.fumbbl.ffb.net.commands.ClientCommandPlayerChoice(
			com.fumbbl.ffb.PlayerChoiceMode.DIVING_TACKLE, new Player<?>[]{tackler}), false);
		com.fumbbl.ffb.model.skill.Skill bt = f.skill(com.fumbbl.ffb.skill.bb2016.BreakTackle.class);
		assertTrue(f.game.getActingPlayer().isSkillUsed(bt), "Break Tackle marked used: it is what made the 3");
		assertEquals(Boolean.TRUE, dodgeMade.published.get(com.fumbbl.ffb.server.step.StepParameterKey.USING_DIVING_TACKLE));
		assertFalse(dodgeMade.reached, "the dodge goes to the tripped label: knocked down, turnover");
	}

	// ── Suspect: a Badly Hurt player left on the pitch (not reproduced) ─────────────────────────

	/**
	 * InjuryResult.applyTo is the only place stock Java writes an injury state onto a player, and it
	 * boxes every KO and casualty. 1,100 current Rust games (khorne/undead/necromantic/nurgle/orc/
	 * human, all three editions) never left a removed player on a pitch square. The 2026-08 khorne
	 * case is not reproducible.
	 */
	@Test
	void notReproduced_badlyHurtIsAlwaysBoxed() throws Exception {
		DefectFixture f = DefectFixture.create("BB2020", "human");
		Player<?> victim = f.away(1);
		f.place(victim, 11, 7, PlayerState.STANDING);
		InjuryResult result = new InjuryResult();
		result.injuryContext().setDefenderId(victim.getId());
		result.injuryContext().setInjury(new PlayerState(PlayerState.BADLY_HURT));
		result.injuryContext().setInjuryType(new com.fumbbl.ffb.injury.Block());
		result.applyTo(new StepEndFuriousOutburst(f.gameState));
		assertEquals(PlayerState.BADLY_HURT, f.game.getFieldModel().getPlayerState(victim).getBase());
		assertFalse(com.fumbbl.ffb.FieldCoordinateBounds.FIELD.isInBounds(
			f.game.getFieldModel().getPlayerCoordinate(victim)), "the Badly Hurt player is in the box, off the pitch");
	}

	// ── JD-006 ──────────────────────────────────────────────────────────────────────────────────

	/**
	 * JD-006: a coach who ends the turn at the Furious Outburst target prompt crashes the step.
	 * StepInitFuriousOutburst's END_TURN branch jumps to END without a TargetSelectionState, and
	 * StepEndFuriousOutburst dereferences it unguarded.
	 */
	@Test
	void jd006_endTurnAtFuriousOutburstPromptThrowsNpe() throws Exception {
		DefectFixture f = DefectFixture.create("BB2025", "human");
		Player<?> outburster = f.home(1);
		f.give(outburster, com.fumbbl.ffb.skill.mixed.special.FuriousOutburst.class);
		f.place(outburster, 10, 7, PlayerState.STANDING);
		f.place(f.away(1), 12, 7, PlayerState.STANDING);   // an eligible target, so the prompt is shown
		f.act(outburster, PlayerAction.FURIOUS_OUTPBURST);

		com.fumbbl.ffb.server.factory.SequenceGeneratorFactory factory =
			f.game.getFactory(FactoryType.Factory.SEQUENCE_GENERATOR);
		FuriousOutburst generator = (FuriousOutburst) factory.forName(SequenceGenerator.Type.FuriousOutburst.name());
		generator.pushSequence(new SequenceGenerator.SequenceParams(f.gameState));
		f.startNextStep();
		assertEquals(StepId.INIT_FURIOUS_OUTBURST, f.current().getId(), "the target prompt is up");
		assertNull(f.game.getFieldModel().getTargetSelectionState());

		NullPointerException npe = assertThrows(NullPointerException.class,
			() -> f.send(new ClientCommandEndTurn(f.game.getTurnMode(), null), true));
		assertTrue(f.current() instanceof StepEndFuriousOutburst, "thrown from StepEndFuriousOutburst");
		assertTrue(Arrays.stream(npe.getStackTrace()).anyMatch(e ->
			e.getClassName().equals(StepEndFuriousOutburst.class.getName())), "stack names the end step");
	}

	// ── JD-008 ──────────────────────────────────────────────────────────────────────────────────

	/**
	 * JD-008: a bb2025 Pitch Invasion that picks a Ball & Chain player. dropPlayer rolls the chain
	 * injury and only PUBLISHES it; the bb2025 kick-off sequence has no step that applies it.
	 */
	@Test
	@SuppressWarnings("unchecked")
	void jd008_pitchInvasionRollsTheChainInjuryButLeavesTheFanaticStanding() throws Exception {
		DefectFixture f = DefectFixture.create("BB2025", "goblin");
		Player<?> fanatic = f.home(6);
		assertTrue(fanatic.hasSkillProperty(com.fumbbl.ffb.model.property.NamedProperties.placedProneCausesInjuryRoll),
			"home #6 is the Fanatic (Ball & Chain)");
		f.place(fanatic, 10, 7, PlayerState.STANDING);

		// 1. The bb2025 kick-off sequence: nothing after APPLY_KICKOFF_RESULT applies an injury.
		com.fumbbl.ffb.server.factory.SequenceGeneratorFactory factory =
			f.game.getFactory(FactoryType.Factory.SEQUENCE_GENERATOR);
		((com.fumbbl.ffb.server.step.generator.Kickoff) factory.forName(SequenceGenerator.Type.Kickoff.name()))
			.pushSequence(new com.fumbbl.ffb.server.step.generator.Kickoff.SequenceParams(f.gameState, false));
		java.util.List<StepId> kickoff = new java.util.ArrayList<>();
		for (com.fumbbl.ffb.server.step.IStep step : f.gameState.getStepStack().toArray()) {
			kickoff.add(step.getId());
		}
		assertTrue(kickoff.contains(StepId.APPLY_KICKOFF_RESULT));
		assertFalse(kickoff.contains(StepId.APOTHECARY), "DEFECT: no APOTHECARY step consumes INJURY_RESULT: " + kickoff);
		f.gameState.getStepStack().clear();

		// 2. The Pitch Invasion picks the Fanatic (the only standing home player: d1 = 1).
		f.dice(1, 6, 6, 6, 6, 6, 6, 6, 6);
		com.fumbbl.ffb.server.step.bb2025.kickoff.StepApplyKickoffResult step =
			new com.fumbbl.ffb.server.step.bb2025.kickoff.StepApplyKickoffResult(f.gameState);
		java.lang.reflect.Method stunPlayers = step.getClass().getDeclaredMethod("stunPlayers",
			com.fumbbl.ffb.model.Team.class, com.fumbbl.ffb.model.FieldModel.class, int.class);
		stunPlayers.setAccessible(true);
		java.util.List<String> affected = (java.util.List<String>) stunPlayers.invoke(step, f.game.getTeamHome(),
			f.game.getFieldModel(), 1);

		System.out.println("JD008 dice: " + f.rolled());
		assertEquals(java.util.Collections.singletonList(fanatic.getId()), affected,
			"the report lists the Fanatic as stunned");
		assertTrue(f.rolled().size() > 1, "the chain injury dice were rolled after the d1 pick: " + f.rolled());
		assertEquals(PlayerState.STANDING, f.game.getFieldModel().getPlayerState(fanatic).getBase(),
			"DEFECT: the Fanatic is reported stunned, its injury was rolled, and it is still STANDING");
	}

	// ── JD-007 ──────────────────────────────────────────────────────────────────────────────────

	/**
	 * JD-007: a bb2025 Stab whose armour is re-rolled by Master Assassin. The re-roll creates the
	 * alternate context; injuryRoll(alternate) rolls the injury on it, but InjuryTypeStab then calls
	 * the context-less setInjury, which interprets the MEMBER (original, unbroken) context.
	 */
	@Test
	void jd007_rerolledStabArmourInterpretsTheOriginalContext() throws Exception {
		DefectFixture f = DefectFixture.create("BB2025", "human");
		Player<?> stabber = f.home(1);
		Player<?> victim = f.away(1);
		f.give(stabber, com.fumbbl.ffb.skill.bb2025.special.MasterAssassin.class);
		f.place(stabber, 10, 7, PlayerState.STANDING);
		f.place(victim, 11, 7, PlayerState.STANDING);
		f.act(stabber, PlayerAction.STAB);

		// armour 1+1 (holds), Master Assassin re-roll 6+6 (breaks), injury 4+4, then spare dice
		f.dice(1, 1, 6, 6, 4, 4, 3, 3, 3, 3);
		InjuryTypeStab stab = new InjuryTypeStab(true);
		InjuryResult result = UtilServerInjury.handleInjury(new StepEndFuriousOutburst(f.gameState), stab,
			stabber, victim, f.game.getFieldModel().getPlayerCoordinate(victim), null, null, ApothecaryMode.DEFENDER);

		InjuryContext original = result.injuryContext();
		InjuryContext alternate = original.getModifiedInjuryContext();
		System.out.println("JD007 dice: " + f.rolled());
		System.out.println("JD007 original: armorBroken=" + original.isArmorBroken()
			+ " armor=" + Arrays.toString(original.getArmorRoll())
			+ " injury=" + Arrays.toString(original.getInjuryRoll())
			+ " casualty=" + Arrays.toString(original.getCasualtyRoll())
			+ " state=" + original.getPlayerState());
		System.out.println("JD007 alternate: " + (alternate == null ? "null" : ("armorBroken=" + alternate.isArmorBroken()
			+ " armor=" + Arrays.toString(alternate.getArmorRoll())
			+ " injury=" + Arrays.toString(alternate.getInjuryRoll())
			+ " casualty=" + Arrays.toString(alternate.getCasualtyRoll())
			+ " state=" + alternate.getPlayerState())));

		assertNotNull(alternate, "Master Assassin produced the alternate (re-rolled armour) context");
		assertTrue(alternate.isArmorBroken());
		assertArrayEquals(new int[]{4, 4}, alternate.getInjuryRoll(), "the injury was rolled on the alternate");
		assertFalse(original.isArmorBroken(), "the original armour held");
		assertNull(original.getInjuryRoll(), "no injury roll on the original");
		assertNotNull(original.getCasualtyRoll(),
			"DEFECT: a casualty roll was made on the ORIGINAL context, whose armour held");
		assertNull(alternate.getPlayerState(),
			"DEFECT: the broken-armour injury roll (4+4) is never interpreted, so the victim is not hurt");
		assertEquals(Arrays.asList("d6=1", "d6=1", "d6=6", "d6=6", "d6=4", "d6=4", "d16=3", "d6=3"), f.rolled(),
			"armour, re-rolled armour, injury -- then a d16+d6 casualty roll the rules never call for");

		// The coach takes the re-roll (StepHandleDropPlayerContext swaps to the alternate) and the
		// result is applied: the victim, armour broken and injury rolled 8 (KO), is still standing.
		StepEndFuriousOutburst anyStep = new StepEndFuriousOutburst(f.gameState);
		result.swapToAlternateContext(anyStep, f.game);
		result.applyTo(anyStep);
		assertEquals(PlayerState.STANDING, f.game.getFieldModel().getPlayerState(victim).getBase(),
			"DEFECT: Master Assassin broke the armour, the injury roll was a KO, and the victim is unhurt");
	}
}
