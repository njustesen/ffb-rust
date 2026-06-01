package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2025.Hatred;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class HatredSkillTest {

    private Hatred skill;

    @BeforeEach
    void setUp() {
        skill = new Hatred();
        skill.postConstruct();
    }

    @Test
    void name_is_Hatred() {
        assertEquals("Hatred", skill.getName());
    }

    @Test
    void category_is_trait() {
        assertEquals(SkillCategory.TRAIT, skill.getCategory());
    }

    @Test
    void has_can_reroll_single_skull_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canRerollSingleSkull));
    }

    @Test
    void is_bb2025_edition() {
        RulesCollection annotation = Hatred.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2025, annotation.value());
    }
}
