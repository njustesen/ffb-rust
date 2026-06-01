package com.fumbbl.ffb.server.util;

import com.fumbbl.ffb.RulesCollection.Rules;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.CsvSource;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class CatchCalcTest {

    // ── minimumRollCatchBB2016 ────────────────────────────────────────────────

    @ParameterizedTest(name = "AG{0} → target {1}")
    @CsvSource({
        "1,6",   // 7-1=6; +0 mod; no -1 bonus (catch vs dodge)
        "2,5",   // 7-2=5
        "3,4",   // 7-3=4
        "4,3",   // 7-4=3
        "5,2",   // 7-5=2
        "6,2",   // 7-6=1 → floor 2
    })
    void bb2016_catch_agility_table(int ag, int expected) {
        assertEquals(expected, CatchCalc.minimumRollCatchBB2016(ag, 0));
    }

    @Test
    void bb2016_catch_harder_than_dodge_for_ag4() {
        // Catch uses base; dodge uses base-1
        int catchTarget = CatchCalc.minimumRollCatchBB2016(4, 0);
        int dodgeTarget = AgilityCalc.minimumRollDodgeBB2016(4, 0);
        assertEquals(catchTarget, dodgeTarget + 1, "catch should be 1 harder than dodge for AG4");
    }

    @Test
    void bb2016_catch_with_positive_modifier_harder() {
        // AG3, +1 rain: 4 + 1 = 5
        assertEquals(5, CatchCalc.minimumRollCatchBB2016(3, 1));
    }

    @Test
    void bb2016_catch_with_negative_modifier_easier() {
        // AG4, -1 sure hands bonus: 3 - 1 = 2 (floor 2)
        assertEquals(2, CatchCalc.minimumRollCatchBB2016(4, -1));
    }

    @Test
    void bb2016_catch_floored_at_2() {
        assertEquals(2, CatchCalc.minimumRollCatchBB2016(6, -10));
    }

    // ── minimumRollInterceptionBB2016 ─────────────────────────────────────────

    @ParameterizedTest(name = "AG{0} → interception target {1}")
    @CsvSource({
        "1,8",   // 6+2=8
        "2,7",   // 5+2=7
        "3,6",   // 4+2=6
        "4,5",   // 3+2=5
        "5,4",   // 2+2=4
        "6,3",   // 1+2=3
    })
    void bb2016_interception_agility_table(int ag, int expected) {
        assertEquals(expected, CatchCalc.minimumRollInterceptionBB2016(ag, 0));
    }

    @Test
    void bb2016_interception_harder_than_catch() {
        for (int ag = 1; ag <= 6; ag++) {
            int intercept = CatchCalc.minimumRollInterceptionBB2016(ag, 0);
            int catch_ = CatchCalc.minimumRollCatchBB2016(ag, 0);
            assertTrue(intercept >= catch_, "interception should be >= catch for AG" + ag);
        }
    }

    // ── minimumRollCatchBB2020 ────────────────────────────────────────────────

    @ParameterizedTest(name = "AG{0} → target {0}")
    @CsvSource({"2,2", "3,3", "4,4", "5,5", "6,6"})
    void bb2020_catch_equals_ag(int ag, int expected) {
        assertEquals(expected, CatchCalc.minimumRollCatchBB2020(ag, 0));
    }

    @Test
    void bb2020_catch_floored_at_2() {
        assertEquals(2, CatchCalc.minimumRollCatchBB2020(1, -5));
    }

    @Test
    void bb2020_catch_with_modifier() {
        // AG3 + rain (+1) = 4
        assertEquals(4, CatchCalc.minimumRollCatchBB2020(3, 1));
    }

    // ── minimumRollInterceptionBB2020 ─────────────────────────────────────────

    @Test
    void bb2020_interception_same_as_catch() {
        // In BB2020 interception has no +2 penalty (unlike BB2016)
        for (int ag = 2; ag <= 6; ag++) {
            assertEquals(
                CatchCalc.minimumRollCatchBB2020(ag, 0),
                CatchCalc.minimumRollInterceptionBB2020(ag, 0),
                "BB2020 interception should equal catch for AG" + ag
            );
        }
    }

    // ── Dispatched methods ────────────────────────────────────────────────────

    @Test
    void dispatched_catch_bb2016_and_bb2020_differ_for_ag4() {
        // BB2016 AG4: catch = 3; BB2020 AG4: catch = 4
        assertEquals(3, CatchCalc.minimumRollCatch(4, 0, Rules.BB2016));
        assertEquals(4, CatchCalc.minimumRollCatch(4, 0, Rules.BB2020));
        assertEquals(4, CatchCalc.minimumRollCatch(4, 0, Rules.BB2025));
    }

    @Test
    void dispatched_interception_bb2016_harder_than_bb2020() {
        // BB2016 AG4: interception = 5; BB2020 AG4: interception = 4
        assertEquals(5, CatchCalc.minimumRollInterception(4, 0, Rules.BB2016));
        assertEquals(4, CatchCalc.minimumRollInterception(4, 0, Rules.BB2020));
    }
}
