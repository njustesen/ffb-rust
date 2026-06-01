package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2016.PilingOn;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class PilingOnSkillTest {

    private PilingOn skill;

    @BeforeEach
    void setUp() {
        skill = new PilingOn();
        skill.postConstruct();
    }

    @Test
    void name_is_Piling_On() {
        assertEquals("Piling On", skill.getName());
    }

    @Test
    void category_is_strength() {
        assertEquals(SkillCategory.STRENGTH, skill.getCategory());
    }

    @Test
    void has_can_pile_on_opponent_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canPileOnOpponent));
    }

    @Test
    void is_bb2016_edition() {
        RulesCollection annotation = PilingOn.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2016, annotation.value());
    }
}
