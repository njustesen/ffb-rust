package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2016.WildAnimal;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class WildAnimalSkillTest {

    private WildAnimal skill;

    @BeforeEach
    void setUp() {
        skill = new WildAnimal();
        skill.postConstruct();
    }

    @Test
    void name_is_Wild_Animal() {
        assertEquals("Wild Animal", skill.getName());
    }

    @Test
    void category_is_extraordinary() {
        assertEquals(SkillCategory.EXTRAORDINARY, skill.getCategory());
    }

    @Test
    void has_needs_to_roll_for_action_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.needsToRollForActionButKeepsTacklezone));
    }

    @Test
    void is_bb2016_edition() {
        RulesCollection annotation = WildAnimal.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2016, annotation.value());
    }
}
