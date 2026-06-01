package com.fumbbl.ffb.server.util;

import com.fumbbl.ffb.BlockResult;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.CsvSource;

import static org.junit.jupiter.api.Assertions.assertEquals;

class BlockResultCalcTest {

    @ParameterizedTest(name = "roll {0} → {1}")
    @CsvSource({
        "1, SKULL",
        "2, BOTH_DOWN",
        "3, PUSHBACK",
        "4, PUSHBACK",
        "5, POW_PUSHBACK",
        "6, POW"
    })
    void blockResultForRoll(int roll, BlockResult expected) {
        assertEquals(expected, BlockResultCalc.blockResultForRoll(roll));
    }

    @Test
    void skull_on_1() {
        assertEquals(BlockResult.SKULL, BlockResultCalc.blockResultForRoll(1));
    }

    @Test
    void bothDown_on_2() {
        assertEquals(BlockResult.BOTH_DOWN, BlockResultCalc.blockResultForRoll(2));
    }

    @Test
    void pushback_on_3() {
        assertEquals(BlockResult.PUSHBACK, BlockResultCalc.blockResultForRoll(3));
    }

    @Test
    void pushback_on_4() {
        assertEquals(BlockResult.PUSHBACK, BlockResultCalc.blockResultForRoll(4));
    }

    @Test
    void powPushback_on_5() {
        assertEquals(BlockResult.POW_PUSHBACK, BlockResultCalc.blockResultForRoll(5));
    }

    @Test
    void pow_on_6() {
        assertEquals(BlockResult.POW, BlockResultCalc.blockResultForRoll(6));
    }
}
