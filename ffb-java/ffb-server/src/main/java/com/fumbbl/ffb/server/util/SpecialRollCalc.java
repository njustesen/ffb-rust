package com.fumbbl.ffb.server.util;

/**
 * Pure special-roll minimum and success calculations extracted from DiceInterpreter.
 * All functions are stateless and require no game context.
 */
public final class SpecialRollCalc {

    // ── Skill minimum rolls ───────────────────────────────────────────────────

    /** Dauntless: must roll >= this on D6 to use own strength instead of being capped. */
    public static int minimumRollDauntless(int attackerStrength, int defenderStrength) {
        return Math.min(6, defenderStrength - attackerStrength + 1);
    }

    /** Tentacles: dodging player escapes on 2D6 >= this. */
    public static int minimumRollTentaclesEscape(int tentaclePlayerStrength, int dodgingPlayerStrength) {
        return 6 + tentaclePlayerStrength - dodgingPlayerStrength;
    }

    /** Shadowing: dodging player escapes on 2D6 >= this. */
    public static int minimumRollShadowingEscape(int shadowingPlayerMovement, int dodgingPlayerMovement) {
        return 8 + shadowingPlayerMovement - dodgingPlayerMovement;
    }

    /** Chainsaw armour break: needs 2+ on D6. */
    public static int minimumRollChainsaw() { return 2; }

    /** Foul Appearance: opposing player must roll 2+ or cannot target this player. */
    public static int minimumRollResistingFoulAppearance() { return 2; }

    /** Confusion / Bone Head / Really Stupid: pass on 2+ (good cond) or 4+ (bad). */
    public static int minimumRollConfusion(boolean goodConditions) { return goodConditions ? 2 : 4; }

    /** Blood Lust (Vampire): needs 2+ or attacks teammate. */
    public static int minimumRollBloodLust() { return 2; }

    /** Animosity: must roll 2+ or refuses to hand off / pass to non-same-race player. */
    public static int minimumRollAnimosity() { return 2; }

    // ── Skill/event success checks ────────────────────────────────────────────

    /** Regeneration: 4+ on D6 brings the player back from SI/death. */
    public static boolean isRegenerationSuccessful(int roll) { return roll >= 4; }

    /** Pitch invasion: player is stunned when roll > 1 AND roll + fameOtherTeam >= 6. */
    public static boolean isAffectedByPitchInvasion(int roll, int fameOtherTeam) {
        return roll > 1 && (roll + fameOtherTeam) >= 6;
    }

    /** Recovering from KO: player wakes up when roll > 1 AND roll + bloodweiserBabes > 3. */
    public static boolean isRecoveringFromKnockout(int roll, int bloodweiserBabes) {
        return roll > 1 && (roll + bloodweiserBabes) > 3;
    }

    /** Always Hungry: on 2+ the player acts; on 1 the ball-carrier is eaten. */
    public static boolean isAlwaysHungrySuccessful(int roll) { return roll >= 2; }

    /** Escape from Always Hungry: on 2+ the player is released unharmed. */
    public static boolean isEscapeFromAlwaysHungrySuccessful(int roll) { return roll >= 2; }

    /** Wild Animal / Exhausted: a roll of 1 means the player fails to act. */
    public static boolean isExhausted(int roll) { return roll == 1; }

    /** Tentacles escape: sum of 2D6 >= minimum. */
    public static boolean isTentaclesEscapeSuccessful(int die1, int die2, int tentacleStr, int dodgingStr) {
        return (die1 + die2) >= minimumRollTentaclesEscape(tentacleStr, dodgingStr);
    }

    /** Shadowing escape: sum of 2D6 >= minimum. */
    public static boolean isShadowingEscapeSuccessful(int die1, int die2, int shadowMov, int dodgingMov) {
        return (die1 + die2) >= minimumRollShadowingEscape(shadowMov, dodgingMov);
    }

    // ── Bribery / post-match events ───────────────────────────────────────────

    /** Bribes: 2+ on D6 avoids the sending-off. */
    public static boolean isBribesSuccessful(int roll) { return roll > 1; }

    /** Argue the Call: 6 on D6 overturns the sending-off. */
    public static boolean isArgueTheCallSuccessful(int roll) { return roll > 5; }

    /** Argue the Call: coach is banned when they roll 1. */
    public static boolean isCoachBanned(int roll) { return roll < 2; }

    /**
     * Stand up from prone: 4+ on D6 (modifier may apply from cards etc.).
     * Unlike a normal skill roll: 1 always fails; 6 does NOT auto-succeed (rule book).
     */
    public static boolean isStandUpSuccessful(int roll, int modifier) {
        return roll > 1 && (roll + modifier) > 3;
    }

    /**
     * Loner / player defecting on Animosity: roll 1–3 = defects (fails).
     */
    public static boolean isPlayerDefecting(int roll) { return roll > 0 && roll < 4; }

    // ── Kickoff events ────────────────────────────────────────────────────────

    /**
     * Riot: D6 < 4 → turn clock advances (+1 turn); D6 >= 4 → goes back (-1 turn).
     * Returns +1 (advance) or -1 (go back).
     */
    public static int interpretRiotRoll(int riotRoll) { return riotRoll < 4 ? 1 : -1; }

    /** True when two dice show the same value (used for doubles detection). */
    public static boolean isDouble(int die1, int die2) { return die1 == die2; }

    private SpecialRollCalc() {}
}
