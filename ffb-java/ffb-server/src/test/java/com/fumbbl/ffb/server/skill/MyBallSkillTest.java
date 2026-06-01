package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.mixed.MyBall;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class MyBallSkillTest {

    private MyBall skill;

    @BeforeEach
    void setUp() {
        skill = new MyBall();
        skill.postConstruct();
    }

    @Test
    void name_is_My_Ball() {
        assertEquals("My Ball", skill.getName());
    }

    @Test
    void category_is_trait() {
        assertEquals(SkillCategory.TRAIT, skill.getCategory());
    }

    @Test
    void has_prevent_regular_hand_over_action_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.preventRegularHandOverAction));
    }

    @Test
    void class_name_is_MyBall() {
        assertEquals("MyBall", skill.getClass().getSimpleName());
    }
}
