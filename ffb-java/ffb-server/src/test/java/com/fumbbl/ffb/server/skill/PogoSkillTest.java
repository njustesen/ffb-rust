package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2025.Pogo;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class PogoSkillTest {

    private Pogo skill;

    @BeforeEach
    void setUp() {
        skill = new Pogo();
        skill.postConstruct();
    }

    @Test
    void name_is_Pogo() {
        assertEquals("Pogo", skill.getName());
    }

    @Test
    void category_is_trait() {
        assertEquals(SkillCategory.TRAIT, skill.getCategory());
    }

    @Test
    void has_can_leap_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canLeap));
    }

    @Test
    void is_bb2025_edition() {
        RulesCollection annotation = Pogo.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2025, annotation.value());
    }
}
