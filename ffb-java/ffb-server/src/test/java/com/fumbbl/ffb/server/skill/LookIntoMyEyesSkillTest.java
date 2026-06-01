package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.mixed.special.LookIntoMyEyes;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class LookIntoMyEyesSkillTest {

    private LookIntoMyEyes skill;

    @BeforeEach
    void setUp() {
        skill = new LookIntoMyEyes();
        skill.postConstruct();
    }

    @Test
    void name_is_Look_Into_My_Eyes() {
        assertEquals("Look Into My Eyes", skill.getName());
    }

    @Test
    void category_is_trait() {
        assertEquals(SkillCategory.TRAIT, skill.getCategory());
    }

    @Test
    void has_can_steal_ball_from_opponent_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canStealBallFromOpponent));
    }

    @Test
    void class_name_is_LookIntoMyEyes() {
        assertEquals("LookIntoMyEyes", skill.getClass().getSimpleName());
    }
}
