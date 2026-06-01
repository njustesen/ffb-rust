package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2016.Chainsaw;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class ChainsawSkillTest {

    private Chainsaw skill;

    @BeforeEach
    void setUp() {
        skill = new Chainsaw();
        skill.postConstruct();
    }

    @Test
    void name_is_Chainsaw() {
        assertEquals("Chainsaw", skill.getName());
    }

    @Test
    void category_is_extraordinary() {
        assertEquals(SkillCategory.EXTRAORDINARY, skill.getCategory());
    }

    @Test
    void has_makes_strength_test_obsolete_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.makesStrengthTestObsolete));
    }

    @Test
    void is_bb2016_edition() {
        RulesCollection annotation = Chainsaw.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2016, annotation.value());
    }
}
