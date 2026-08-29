package com.fumbbl.ffb.ai.parity.heuristic;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Pins {@link HeuristicDriver#useSkill} against Rust's {@code AgentPrompt::SkillUse} arm — both the
 * weight it picks with and, just as importantly, what it COSTS.
 *
 * <p>This class was unreachable for the whole lineman campaign: a team with no skills is never
 * offered one. So Rust scored the prompt and Java had no arm for it, answering through the random
 * contract's fixed "always use" rule instead. That rule is free; Rust's costs two sampler draws.
 * The amazons carry Dodge on every player, and the first failed dodge therefore knocked the two
 * random streams out of step — after which the agents enumerated identical candidate lists with
 * identical weights and still picked differently, because they were reading different numbers.
 *
 * <p>Hence the two halves below. The draw count is not a detail of the implementation; it is the
 * contract, and it is the half that actually broke.
 */
class SkillUseTest {

    private static HeuristicDriver driver(float tempScale) {
        return new HeuristicDriver(21L, tempScale, ClassMask.ALL);
    }

    /** Rust's table, verbatim. Anything unlisted is a 0.50 coin flip. */
    @Test
    void argmaxFollowsRustsWeightTable() {
        // At temperature 0 the sampler is an argmax, so the answer is simply "is w > 0.5".
        HeuristicDriver d = driver(0.0f);
        assertTrue(d.useSkill("Dodge"), "Dodge 0.95");
        assertTrue(d.useSkill("Juggernaut"), "Juggernaut 0.80");
        assertTrue(d.useSkill("HitAndRun"), "HitAndRun 0.70");
        assertTrue(d.useSkill("Fend"), "Fend 0.85");
        assertTrue(d.useSkill("Wrestle"), "Wrestle 0.55");
        assertTrue(d.useSkill("QuickBite"), "QuickBite 0.85");
        assertTrue(d.useSkill("AnimalSavagery"), "AnimalSavagery 0.85");
        // 0.50 against 0.50: Rust's argmax keeps the FIRST maximum, which is "use".
        assertTrue(d.useSkill("SomethingNobodyHasTabulated"), "unlisted defaults to 0.50");
    }

    /**
     * The half that broke. Rust's {@code pick} spends one draw on the epsilon test and one on the
     * cumulative walk whenever there is more than one option and the temperature is live; at
     * temperature 0 it argmaxes and spends nothing.
     */
    @Test
    void drawCostMatchesRust() {
        for (float scale : new float[] {1.0f, 1.0e6f}) {
            HeuristicDriver d = driver(scale);
            long before = d.sampler().drawCount();
            d.useSkill("Dodge");
            assertEquals(2L, d.sampler().drawCount() - before,
                "live temperature must spend exactly two draws @ " + scale);
        }
        HeuristicDriver argmax = driver(0.0f);
        long before = argmax.sampler().drawCount();
        argmax.useSkill("Dodge");
        assertEquals(0L, argmax.sampler().drawCount() - before, "argmax draws nothing");
    }

    /**
     * A 0.95/0.05 split must actually decline sometimes, or the sampled arm is indistinguishable
     * from the fixed "always use" rule it replaced — which is exactly the bug that hid here.
     */
    @Test
    void sampledDodgeIsNotAlwaysUse() {
        HeuristicDriver d = driver(1.0f);
        int declines = 0;
        for (int i = 0; i < 400; i++) {
            if (!d.useSkill("Dodge")) {
                declines++;
            }
        }
        assertTrue(declines > 0, "0.95 vs 0.05 never declined in 400 draws");
        assertTrue(declines < 200, "declined " + declines + "/400; the weights are not being read");
    }
}
