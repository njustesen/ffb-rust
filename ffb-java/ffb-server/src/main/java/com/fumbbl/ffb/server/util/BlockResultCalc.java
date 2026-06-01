package com.fumbbl.ffb.server.util;

import com.fumbbl.ffb.BlockResult;

public final class BlockResultCalc {

    public static BlockResult blockResultForRoll(int roll) {
        switch (roll) {
        case 1: return BlockResult.SKULL;
        case 2: return BlockResult.BOTH_DOWN;
        case 5: return BlockResult.POW_PUSHBACK;
        case 6: return BlockResult.POW;
        default: return BlockResult.PUSHBACK;
        }
    }

    private BlockResultCalc() {}
}
