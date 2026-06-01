package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.mixed.OnTheBall;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class OnTheBallSkillTest {

    private OnTheBall skill;

    @BeforeEach
    void setUp() {
        skill = new OnTheBall();
        skill.postConstruct();
    }

    @Test
    void name_is_On_The_Ball() {
        assertEquals("On The Ball", skill.getName());
    }

    @Test
    void category_is_passing() {
        assertEquals(SkillCategory.PASSING, skill.getCategory());
    }

    @Test
    void has_can_move_during_kickoff_scatter_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canMoveDuringKickOffScatter));
    }

    @Test
    void class_name_is_OnTheBall() {
        assertEquals("OnTheBall", skill.getClass().getSimpleName());
    }
}
