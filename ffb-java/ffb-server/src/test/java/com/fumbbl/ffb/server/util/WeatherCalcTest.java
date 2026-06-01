package com.fumbbl.ffb.server.util;

import com.fumbbl.ffb.Weather;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.CsvSource;

import static org.junit.jupiter.api.Assertions.*;

class WeatherCalcTest {

    @ParameterizedTest(name = "roll {0} → {1}")
    @CsvSource({
        "2,  SWELTERING_HEAT",
        "3,  VERY_SUNNY",
        "4,  NICE",
        "5,  NICE",
        "6,  NICE",
        "7,  NICE",
        "8,  NICE",
        "9,  NICE",
        "10, NICE",
        "11, POURING_RAIN",
        "12, BLIZZARD"
    })
    void weatherForRoll(int total, Weather expected) {
        assertEquals(expected, WeatherCalc.weatherForRoll(total));
    }
}
