package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.mixed.Pro;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class ProSkillTest {

    private Pro skill;

    @BeforeEach
    void setUp() {
        skill = new Pro();
        skill.postConstruct();
    }

    @Test
    void name_is_Pro() {
        assertEquals("Pro", skill.getName());
    }

    @Test
    void category_is_general() {
        assertEquals(SkillCategory.GENERAL, skill.getCategory());
    }

    @Test
    void has_can_reroll_once_per_turn_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canRerollOncePerTurn));
    }

    @Test
    void class_name_is_Pro() {
        assertEquals("Pro", skill.getClass().getSimpleName());
    }
}
