package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2025.Unsteady;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class UnsteadySkillTest {

    private Unsteady skill;

    @BeforeEach
    void setUp() {
        skill = new Unsteady();
        skill.postConstruct();
    }

    @Test
    void name_is_Unsteady() {
        assertEquals("Unsteady", skill.getName());
    }

    @Test
    void category_is_trait() {
        assertEquals(SkillCategory.TRAIT, skill.getCategory());
    }

    @Test
    void has_prevent_secure_the_ball_action_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.preventSecureTheBallAction));
    }

    @Test
    void is_bb2025_edition() {
        RulesCollection annotation = Unsteady.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2025, annotation.value());
    }
}
