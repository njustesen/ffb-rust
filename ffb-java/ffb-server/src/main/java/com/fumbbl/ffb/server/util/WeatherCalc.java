package com.fumbbl.ffb.server.util;

import com.fumbbl.ffb.Weather;

/**
 * Pure weather table calculation.
 * Mirrors Java DiceInterpreter.interpretWeather().
 */
public final class WeatherCalc {

    /**
     * Map a 2D6 sum to the resulting weather.
     * 2=SWELTERING_HEAT, 3=VERY_SUNNY, 4–10=NICE, 11=POURING_RAIN, 12=BLIZZARD.
     */
    public static Weather weatherForRoll(int total) {
        switch (total) {
        case 2:  return Weather.SWELTERING_HEAT;
        case 3:  return Weather.VERY_SUNNY;
        case 11: return Weather.POURING_RAIN;
        case 12: return Weather.BLIZZARD;
        default: return Weather.NICE;
        }
    }

    private WeatherCalc() {}
}
