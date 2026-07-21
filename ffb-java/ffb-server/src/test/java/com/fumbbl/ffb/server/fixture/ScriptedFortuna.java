package com.fumbbl.ffb.server.fixture;

import com.fumbbl.ffb.server.util.rng.Fortuna;

import java.util.ArrayDeque;
import java.util.Deque;

/**
 * A deterministic {@link Fortuna} for step tests: instead of drawing from the
 * cryptographic entropy pool, it returns a caller-supplied sequence of die
 * faces. This mirrors how the Rust step tests seed their RNG
 * ({@code ffb-rust/crates/ffb-engine} tests build a scripted dice source and
 * assert the exact resulting step outcome).
 *
 * <p>Every game die in the Java engine ultimately resolves through
 * {@link com.fumbbl.ffb.server.DiceRoller}, whose {@code rollDice(...)} methods
 * end at {@code gameState.getServer().getFortuna().getDieRoll(sides)}. By
 * overriding {@link #getDieRoll(int)} on the single {@code Fortuna} the test
 * server hands out, <em>all</em> categories become deterministic: plain d3/d4/d6/
 * d8/d16 draws, {@code rollWeather()}/{@code rollArmour()}/{@code rollInjury()}
 * (2d6 = two draws), block dice ({@code rollBlockDice} -> d6 faces 1..6 mapping
 * skull..pow) and scatter/throw-in direction ({@code rollScatterDirection} -> d8).
 *
 * <p>Scripted faces are consumed in draw order. Values must be within
 * {@code 1..sides} for the die being rolled or an {@link IllegalStateException}
 * is thrown (a scripted 7 requested as a d6 is a test bug, not a silent skip -
 * unlike {@code DiceRoller.addTestRoll}). When the script is exhausted the
 * generator falls back to the real random {@link Fortuna} behaviour, so
 * incidental background rolls a test does not care about still work.
 *
 * <h3>Constraints on determinism</h3>
 * <ul>
 *   <li><b>Sequential, not categorised.</b> One flat queue feeds every die.
 *       Script faces in the exact order the step under test rolls them; an
 *       unexpected earlier roll (e.g. a Pro/Loner re-roll, Dauntless, or a
 *       leader/entropy draw) will consume a face you intended for a later
 *       roll.</li>
 *   <li><b>Re-rolls draw again.</b> A re-rolled dodge/GFI/block consumes the
 *       next scripted face(s); provide faces for both the initial roll and the
 *       re-roll.</li>
 *   <li><b>Direction is a raw d8</b> (1=N..8=NW for the home team; the away team
 *       mirroring happens above the RNG in {@code DirectionDiceCategory}). Block
 *       dice are raw d6 with the BlockEnums mapping 1=skull,2=bothdown,
 *       3/4=pushback,5=stumble,6=pow.</li>
 * </ul>
 */
public class ScriptedFortuna extends Fortuna {

	private final Deque<Integer> scriptedRolls = new ArrayDeque<>();

	/** Append die faces to the script, consumed in order by later draws. */
	public void script(int... rolls) {
		for (int roll : rolls) {
			scriptedRolls.addLast(roll);
		}
	}

	/** Discard any remaining scripted faces (reverts to real random draws). */
	public void clearScript() {
		scriptedRolls.clear();
	}

	/** Number of scripted faces not yet consumed. */
	public int remaining() {
		return scriptedRolls.size();
	}

	@Override
	public int getDieRoll(int sides) {
		if (scriptedRolls.isEmpty()) {
			return super.getDieRoll(sides);
		}
		int roll = scriptedRolls.removeFirst();
		if (roll < 1 || roll > sides) {
			throw new IllegalStateException(
				"Scripted roll " + roll + " is out of range for a d" + sides + " draw");
		}
		return roll;
	}
}
