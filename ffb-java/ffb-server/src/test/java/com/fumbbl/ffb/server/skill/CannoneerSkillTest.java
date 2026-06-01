package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.skill.mixed.Cannoneer;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class CannoneerSkillTest {

    private Cannoneer skill;

    @BeforeEach
    void setUp() {
        skill = new Cannoneer();
        skill.postConstruct();
    }

    @Test
    void name_is_Cannoneer() {
        assertEquals("Cannoneer", skill.getName());
    }

    @Test
    void category_is_passing() {
        assertEquals(SkillCategory.PASSING, skill.getCategory());
    }

    @Test
    void skill_properties_are_not_null() {
        assertNotNull(skill.getSkillProperties());
    }

    @Test
    void class_name_is_Cannoneer() {
        assertEquals("Cannoneer", skill.getClass().getSimpleName());
    }
}
