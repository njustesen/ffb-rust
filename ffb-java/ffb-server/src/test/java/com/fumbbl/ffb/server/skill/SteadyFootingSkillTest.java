package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2025.SteadyFooting;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class SteadyFootingSkillTest {

    private SteadyFooting skill;

    @BeforeEach
    void setUp() {
        skill = new SteadyFooting();
        skill.postConstruct();
    }

    @Test
    void name_is_Steady_Footing() {
        assertEquals("Steady Footing", skill.getName());
    }

    @Test
    void category_is_trait() {
        assertEquals(SkillCategory.TRAIT, skill.getCategory());
    }

    @Test
    void has_can_avoid_falling_down_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canAvoidFallingDown));
    }

    @Test
    void is_bb2025_edition() {
        RulesCollection annotation = SteadyFooting.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2025, annotation.value());
    }
}
