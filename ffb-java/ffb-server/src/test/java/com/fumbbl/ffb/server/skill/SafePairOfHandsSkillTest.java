package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.mixed.SafePairOfHands;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class SafePairOfHandsSkillTest {

    private SafePairOfHands skill;

    @BeforeEach
    void setUp() {
        skill = new SafePairOfHands();
        skill.postConstruct();
    }

    @Test
    void name_is_Safe_Pair_Of_Hands() {
        assertEquals("Safe Pair Of Hands", skill.getName());
    }

    @Test
    void category_is_agility() {
        assertEquals(SkillCategory.AGILITY, skill.getCategory());
    }

    @Test
    void has_can_place_ball_when_knocked_down_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canPlaceBallWhenKnockedDownOrPlacedProne));
    }

    @Test
    void class_name_is_SafePairOfHands() {
        assertEquals("SafePairOfHands", skill.getClass().getSimpleName());
    }
}
