package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.skill.mixed.AgilityIncrease;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class AgilityIncreaseSkillTest {

    private AgilityIncrease skill;

    @BeforeEach
    void setUp() {
        skill = new AgilityIncrease();
        skill.postConstruct();
    }

    @Test
    void name_is_plus_AG() {
        assertEquals("+AG", skill.getName());
    }

    @Test
    void category_is_stat_increase() {
        assertEquals(SkillCategory.STAT_INCREASE, skill.getCategory());
    }

    @Test
    void skill_properties_are_not_null() {
        assertNotNull(skill.getSkillProperties());
    }

    @Test
    void class_name_is_AgilityIncrease() {
        assertEquals("AgilityIncrease", skill.getClass().getSimpleName());
    }
}
