package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2016.Regeneration;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class RegenerationSkillTest {

    private Regeneration skill;

    @BeforeEach
    void setUp() {
        skill = new Regeneration();
        skill.postConstruct();
    }

    @Test
    void name_is_Regeneration() {
        assertEquals("Regeneration", skill.getName());
    }

    @Test
    void category_is_extraordinary() {
        assertEquals(SkillCategory.EXTRAORDINARY, skill.getCategory());
    }

    @Test
    void has_can_roll_to_save_from_injury_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canRollToSaveFromInjury));
    }

    @Test
    void is_bb2016_edition() {
        RulesCollection annotation = Regeneration.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2016, annotation.value());
    }
}
