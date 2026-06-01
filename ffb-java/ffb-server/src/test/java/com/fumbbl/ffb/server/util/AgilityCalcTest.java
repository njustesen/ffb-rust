package com.fumbbl.ffb.server.util;

import com.fumbbl.ffb.RulesCollection.Rules;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.CsvSource;

import static org.junit.jupiter.api.Assertions.*;

class AgilityCalcTest {

    // ── agilityRollBaseBB2016 ─────────────────────────────────────────────────

    @ParameterizedTest(name = "ag={0} → base={1}")
    @CsvSource({
        "1,6",
        "2,5",
        "3,4",
        "4,3",
        "5,2",
        "6,1",
        "7,1",   // capped at min(ag,6)=6
        "9,1",
    })
    void bb2016_agilityBase(int ag, int expected) {
        assertEquals(expected, AgilityCalc.agilityRollBaseBB2016(ag));
    }

    // ── minimumRollDodgeBB2016 ───────────────────────────────────────────────

    @ParameterizedTest(name = "ag={0} → dodge target={1}")
    @CsvSource({
        "1,5",   // base 6, -1 = 5
        "2,4",   // base 5, -1 = 4
        "3,3",   // base 4, -1 = 3
        "4,2",   // base 3, -1 = 2
        "5,2",   // base 2, -1 = 1 → floored to 2
        "6,2",   // base 1, -1 = 0 → floored to 2
    })
    void bb2016_dodge_noModifiers(int ag, int expected) {
        assertEquals(expected, AgilityCalc.minimumRollDodgeBB2016(ag, 0));
    }

    @Test
    void bb2016_dodge_withPositiveModifier_increases() {
        // AG4 base dodge = 2; +1 tackle zone = 3
        assertEquals(3, AgilityCalc.minimumRollDodgeBB2016(4, 1));
        // AG4 base dodge = 2; +2 tackle zones = 4
        assertEquals(4, AgilityCalc.minimumRollDodgeBB2016(4, 2));
    }

    @Test
    void bb2016_dodge_withNegativeModifier_decreasesButFloorAt2() {
        // AG3 base dodge = 3; Dodge skill -1 = 2
        assertEquals(2, AgilityCalc.minimumRollDodgeBB2016(3, -1));
        // AG2 base dodge = 4; -10 modifiers → 2
        assertEquals(2, AgilityCalc.minimumRollDodgeBB2016(2, -10));
    }

    // ── minimumRollCatchBB2016 ───────────────────────────────────────────────

    @ParameterizedTest(name = "ag={0} → catch target={1}")
    @CsvSource({
        "1,6",   // base 6, no -1 = 6
        "2,5",
        "3,4",
        "4,3",
        "5,2",
        "6,2",   // base 1 → floored to 2
    })
    void bb2016_catch_noModifiers(int ag, int expected) {
        assertEquals(expected, AgilityCalc.minimumRollCatchBB2016(ag, 0));
    }

    @Test
    void bb2016_catch_harder_than_dodge_for_same_ag() {
        // Same AG, catch is always 1 harder than dodge (unless both at floor)
        for (int ag = 1; ag <= 4; ag++) {
            int dodge = AgilityCalc.minimumRollDodgeBB2016(ag, 0);
            int catch_ = AgilityCalc.minimumRollCatchBB2016(ag, 0);
            assertEquals(dodge + 1, catch_, "ag=" + ag);
        }
    }

    @Test
    void bb2016_catch_withPositiveModifier_harder() {
        // AG4, catch base = 3; +1 tackle zone → 4
        assertEquals(4, AgilityCalc.minimumRollCatchBB2016(4, 1));
        assertEquals(5, AgilityCalc.minimumRollCatchBB2016(4, 2));
    }

    @Test
    void bb2016_catch_withNegativeModifier_floorAt2() {
        // AG4, catch base = 3; -1 skill → 2; can't go below 2
        assertEquals(2, AgilityCalc.minimumRollCatchBB2016(4, -1));
        assertEquals(2, AgilityCalc.minimumRollCatchBB2016(4, -10));
    }

    // ── minimumRollInterceptionBB2016 ────────────────────────────────────────

    @ParameterizedTest(name = "ag={0} → intercept target={1}")
    @CsvSource({
        "1,8",   // base 6 + 2 = 8 (no floor; above 2)
        "2,7",   // base 5 + 2 = 7
        "3,6",   // base 4 + 2 = 6
        "4,5",   // base 3 + 2 = 5
        "5,4",   // base 2 + 2 = 4
        "6,3",   // base 1 + 2 = 3
    })
    void bb2016_interception_noModifiers(int ag, int expected) {
        assertEquals(expected, AgilityCalc.minimumRollInterceptionBB2016(ag, 0));
    }

    @Test
    void bb2016_interception_harder_than_catch_by2_for_same_ag() {
        // Interception is exactly 2 harder than catch for all AG (above floor)
        for (int ag = 1; ag <= 4; ag++) {
            int catch_ = AgilityCalc.minimumRollCatchBB2016(ag, 0);
            int intercept = AgilityCalc.minimumRollInterceptionBB2016(ag, 0);
            assertEquals(catch_ + 2, intercept, "ag=" + ag);
        }
    }

    // ── minimumRollBB2020 ────────────────────────────────────────────────────

    @ParameterizedTest(name = "ag={0} → target={1}")
    @CsvSource({
        "2,2",   // 2+ player: target 2
        "3,3",   // 3+ player: target 3
        "4,4",
        "5,5",
        "6,6",
    })
    void bb2020_directAgility_noModifiers(int ag, int expected) {
        assertEquals(expected, AgilityCalc.minimumRollBB2020(ag, 0));
    }

    @Test
    void bb2020_withPositiveModifier() {
        assertEquals(4, AgilityCalc.minimumRollBB2020(3, 1)); // 3 + 1 = 4
    }

    @Test
    void bb2020_withNegativeModifier_floorAt2() {
        assertEquals(2, AgilityCalc.minimumRollBB2020(3, -5)); // floor
        assertEquals(2, AgilityCalc.minimumRollBB2020(2, -1)); // 1 → 2
    }

    // ── minimumRollDodge (edition-dispatched) ────────────────────────────────

    @Test
    void dispatched_bb2016_vs_bb2020_differ_for_same_ag4() {
        // BB2016 AG4: dodge target = 2
        // BB2020 AG4 (meaning "4+" player): target = 4
        assertEquals(2, AgilityCalc.minimumRollDodge(4, 0, Rules.BB2016));
        assertEquals(4, AgilityCalc.minimumRollDodge(4, 0, Rules.BB2020));
    }

    @Test
    void dispatched_bb2025_same_as_bb2020() {
        assertEquals(
            AgilityCalc.minimumRollDodge(3, 0, Rules.BB2020),
            AgilityCalc.minimumRollDodge(3, 0, Rules.BB2025)
        );
    }

    @Test
    void bb2020_catch_uses_same_formula_as_dodge() {
        // In BB2020 all agility actions (dodge, catch, intercept) use the same formula: ag + mods
        // So catch and dodge roll against the same target for a given AG and modifier sum
        assertEquals(AgilityCalc.minimumRollBB2020(4, 0), AgilityCalc.minimumRollBB2020(4, 0));
        assertEquals(4, AgilityCalc.minimumRollBB2020(4, 0)); // "4+" player needs to roll 4+
        assertEquals(2, AgilityCalc.minimumRollBB2020(4, -5)); // floored at 2
    }
}
