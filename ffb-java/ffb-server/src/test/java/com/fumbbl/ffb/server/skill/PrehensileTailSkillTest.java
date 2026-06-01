package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2016.PrehensileTail;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class PrehensileTailSkillTest {

    private PrehensileTail skill;

    @BeforeEach
    void setUp() {
        skill = new PrehensileTail();
        skill.postConstruct();
    }

    @Test
    void name_is_Prehensile_Tail() {
        assertEquals("Prehensile Tail", skill.getName());
    }

    @Test
    void category_is_mutation() {
        assertEquals(SkillCategory.MUTATION, skill.getCategory());
    }

    @Test
    void has_makes_dodging_harder_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.makesDodgingHarder));
    }

    @Test
    void is_bb2016_edition() {
        RulesCollection annotation = PrehensileTail.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2016, annotation.value());
    }
}
