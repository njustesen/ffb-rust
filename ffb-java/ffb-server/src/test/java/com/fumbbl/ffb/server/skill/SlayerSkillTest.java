package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.skill.mixed.special.Slayer;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class SlayerSkillTest {

    private Slayer skill;

    @BeforeEach
    void setUp() {
        skill = new Slayer();
        skill.postConstruct();
    }

    @Test
    void name_is_Slayer() {
        assertEquals("Slayer", skill.getName());
    }

    @Test
    void category_is_trait() {
        assertEquals(SkillCategory.TRAIT, skill.getCategory());
    }

    @Test
    void skill_properties_are_not_null() {
        assertNotNull(skill.getSkillProperties());
    }

    @Test
    void class_name_is_Slayer() {
        assertEquals("Slayer", skill.getClass().getSimpleName());
    }
}
