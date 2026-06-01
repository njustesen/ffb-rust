package com.fumbbl.ffb.server.model;

import com.fumbbl.ffb.model.RosterPlayer;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class PlayerModelTest {

    @Test
    void player_name_is_set_and_retrieved() {
        RosterPlayer p = new RosterPlayer();
        p.setName("Joe Bloggs");
        assertEquals("Joe Bloggs", p.getName());
    }

    @Test
    void player_nr_is_set_and_retrieved() {
        RosterPlayer p = new RosterPlayer();
        p.setNr(7);
        assertEquals(7, p.getNr());
    }

    @Test
    void player_movement_is_set_and_retrieved() {
        RosterPlayer p = new RosterPlayer();
        p.setMovement(6);
        assertEquals(6, p.getMovement());
    }

    @Test
    void player_strength_is_set_and_retrieved() {
        RosterPlayer p = new RosterPlayer();
        p.setStrength(3);
        assertEquals(3, p.getStrength());
    }

    @Test
    void player_agility_is_set_and_retrieved() {
        RosterPlayer p = new RosterPlayer();
        p.setAgility(3);
        assertEquals(3, p.getAgility());
    }
}
