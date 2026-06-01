package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.mixed.Leader;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class LeaderSkillTest {

    private Leader skill;

    @BeforeEach
    void setUp() {
        skill = new Leader();
        skill.postConstruct();
    }

    @Test
    void name_is_Leader() {
        assertEquals("Leader", skill.getName());
    }

    @Test
    void category_is_passing() {
        assertEquals(SkillCategory.PASSING, skill.getCategory());
    }

    @Test
    void has_grants_team_reroll_when_on_pitch_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.grantsTeamReRollWhenOnPitch));
    }

    @Test
    void class_name_is_Leader() {
        assertEquals("Leader", skill.getClass().getSimpleName());
    }
}
