package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.mixed.special.BalefulHex;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class BalefulHexSkillTest {

    private BalefulHex skill;

    @BeforeEach
    void setUp() {
        skill = new BalefulHex();
        skill.postConstruct();
    }

    @Test
    void name_is_Baleful_Hex() {
        assertEquals("Baleful Hex", skill.getName());
    }

    @Test
    void category_is_trait() {
        assertEquals(SkillCategory.TRAIT, skill.getCategory());
    }

    @Test
    void has_can_make_opponent_miss_turn_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canMakeOpponentMissTurn));
    }

    @Test
    void class_name_is_BalefulHex() {
        assertEquals("BalefulHex", skill.getClass().getSimpleName());
    }
}
