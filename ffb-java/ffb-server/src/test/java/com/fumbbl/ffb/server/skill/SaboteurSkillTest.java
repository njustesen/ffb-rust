package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2025.Saboteur;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class SaboteurSkillTest {

    private Saboteur skill;

    @BeforeEach
    void setUp() {
        skill = new Saboteur();
        skill.postConstruct();
    }

    @Test
    void name_is_Saboteur() {
        assertEquals("Saboteur", skill.getName());
    }

    @Test
    void category_is_devious() {
        assertEquals(SkillCategory.DEVIOUS, skill.getCategory());
    }

    @Test
    void has_can_sabotage_blocker_on_knockdown_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canSabotageBlockerOnKnockdown));
    }

    @Test
    void is_bb2025_edition() {
        RulesCollection annotation = Saboteur.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2025, annotation.value());
    }
}
