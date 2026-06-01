package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2025.Punt;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class PuntSkillTest {

    private Punt skill;

    @BeforeEach
    void setUp() {
        skill = new Punt();
        skill.postConstruct();
    }

    @Test
    void name_is_Punt() {
        assertEquals("Punt", skill.getName());
    }

    @Test
    void category_is_passing() {
        assertEquals(SkillCategory.PASSING, skill.getCategory());
    }

    @Test
    void has_can_punt_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canPunt));
    }

    @Test
    void is_bb2025_edition() {
        RulesCollection annotation = Punt.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2025, annotation.value());
    }
}
