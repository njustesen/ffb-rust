package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.mixed.Kick;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class KickSkillTest {

    private Kick skill;

    @BeforeEach
    void setUp() {
        skill = new Kick();
        skill.postConstruct();
    }

    @Test
    void name_is_Kick() {
        assertEquals("Kick", skill.getName());
    }

    @Test
    void category_is_general() {
        assertEquals(SkillCategory.GENERAL, skill.getCategory());
    }

    @Test
    void has_can_reduce_kick_distance_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canReduceKickDistance));
    }

    @Test
    void has_skill_properties_not_null() {
        assertNotNull(skill.getSkillProperties());
    }
}
