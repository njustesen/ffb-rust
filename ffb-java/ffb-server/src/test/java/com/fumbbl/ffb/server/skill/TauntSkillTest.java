package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2025.Taunt;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class TauntSkillTest {

    private Taunt skill;

    @BeforeEach
    void setUp() {
        skill = new Taunt();
        skill.postConstruct();
    }

    @Test
    void name_is_Taunt() {
        assertEquals("Taunt", skill.getName());
    }

    @Test
    void category_is_general() {
        assertEquals(SkillCategory.GENERAL, skill.getCategory());
    }

    @Test
    void has_force_opponent_to_follow_up_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.forceOpponentToFollowUp));
    }

    @Test
    void is_bb2025_edition() {
        RulesCollection annotation = Taunt.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2025, annotation.value());
    }
}
