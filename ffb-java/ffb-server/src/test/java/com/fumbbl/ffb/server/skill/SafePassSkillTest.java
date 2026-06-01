package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.mixed.SafePass;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class SafePassSkillTest {

    private SafePass skill;

    @BeforeEach
    void setUp() {
        skill = new SafePass();
        skill.postConstruct();
    }

    @Test
    void name_is_Safe_Pass() {
        assertEquals("Safe Pass", skill.getName());
    }

    @Test
    void category_is_passing() {
        assertEquals(SkillCategory.PASSING, skill.getCategory());
    }

    @Test
    void has_dont_drop_fumbles_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.dontDropFumbles));
    }

    @Test
    void class_name_is_SafePass() {
        assertEquals("SafePass", skill.getClass().getSimpleName());
    }
}
