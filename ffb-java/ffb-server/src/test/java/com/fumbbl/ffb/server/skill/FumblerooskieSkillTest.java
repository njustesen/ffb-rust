package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2020.Fumblerooskie;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class FumblerooskieSkillTest {

    private Fumblerooskie skill;

    @BeforeEach
    void setUp() {
        skill = new Fumblerooskie();
        skill.postConstruct();
    }

    @Test
    void name_is_Fumblerooskie() {
        assertEquals("Fumblerooskie", skill.getName());
    }

    @Test
    void category_is_passing() {
        assertEquals(SkillCategory.PASSING, skill.getCategory());
    }

    @Test
    void has_can_drop_ball_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canDropBall));
    }

    @Test
    void is_bb2020_edition() {
        RulesCollection annotation = Fumblerooskie.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2020, annotation.value());
    }
}
