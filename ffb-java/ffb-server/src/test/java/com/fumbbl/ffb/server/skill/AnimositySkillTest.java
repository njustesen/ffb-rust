package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2025.Animosity;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class AnimositySkillTest {

    private Animosity skill;

    @BeforeEach
    void setUp() {
        skill = new Animosity();
        skill.postConstruct();
    }

    @Test
    void name_is_Animosity() {
        assertEquals("Animosity", skill.getName());
    }

    @Test
    void category_is_trait() {
        assertEquals(SkillCategory.TRAIT, skill.getCategory());
    }

    @Test
    void has_has_to_roll_to_pass_ball_on_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.hasToRollToPassBallOn));
    }

    @Test
    void is_bb2025_edition() {
        RulesCollection annotation = Animosity.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2025, annotation.value());
    }
}
