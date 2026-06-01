package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.mixed.PickMeUp;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class PickMeUpSkillTest {

    private PickMeUp skill;

    @BeforeEach
    void setUp() {
        skill = new PickMeUp();
        skill.postConstruct();
    }

    @Test
    void name_is_Pick_me_up() {
        assertEquals("Pick-me-up", skill.getName());
    }

    @Test
    void category_is_trait() {
        assertEquals(SkillCategory.TRAIT, skill.getCategory());
    }

    @Test
    void has_can_stand_up_team_mates_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canStandUpTeamMates));
    }

    @Test
    void class_name_is_PickMeUp() {
        assertEquals("PickMeUp", skill.getClass().getSimpleName());
    }
}
