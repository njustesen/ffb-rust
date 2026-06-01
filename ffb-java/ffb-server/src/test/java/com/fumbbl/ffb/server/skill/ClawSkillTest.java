package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2016.Claw;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class ClawSkillTest {

    private Claw skill;

    @BeforeEach
    void setUp() {
        skill = new Claw();
        skill.postConstruct();
    }

    @Test
    void name_is_Claw() {
        assertEquals("Claw", skill.getName());
    }

    @Test
    void category_is_mutation() {
        assertEquals(SkillCategory.MUTATION, skill.getCategory());
    }

    @Test
    void has_reduces_armour_to_fixed_value_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.reducesArmourToFixedValue));
    }

    @Test
    void is_bb2016_edition() {
        RulesCollection annotation = Claw.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2016, annotation.value());
    }
}
