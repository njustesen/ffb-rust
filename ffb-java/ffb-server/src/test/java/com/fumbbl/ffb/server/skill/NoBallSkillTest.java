package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2025.NoBall;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class NoBallSkillTest {

    private NoBall skill;

    @BeforeEach
    void setUp() {
        skill = new NoBall();
        skill.postConstruct();
    }

    @Test
    void name_is_No_Ball() {
        assertEquals("No Ball", skill.getName());
    }

    @Test
    void category_is_trait() {
        assertEquals(SkillCategory.TRAIT, skill.getCategory());
    }

    @Test
    void has_prevent_catch_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.preventCatch));
    }

    @Test
    void is_bb2025_edition() {
        RulesCollection annotation = NoBall.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2025, annotation.value());
    }
}
