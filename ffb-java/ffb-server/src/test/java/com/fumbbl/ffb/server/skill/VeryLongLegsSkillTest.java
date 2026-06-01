package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.skill.bb2016.VeryLongLegs;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class VeryLongLegsSkillTest {

    private VeryLongLegs skill;

    @BeforeEach
    void setUp() {
        skill = new VeryLongLegs();
        skill.postConstruct();
    }

    @Test
    void name_is_Very_Long_Legs() {
        assertEquals("Very Long Legs", skill.getName());
    }

    @Test
    void category_is_mutation() {
        assertEquals(SkillCategory.MUTATION, skill.getCategory());
    }

    @Test
    void skill_properties_are_not_null() {
        assertNotNull(skill.getSkillProperties());
    }

    @Test
    void is_bb2016_edition() {
        RulesCollection annotation = VeryLongLegs.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2016, annotation.value());
    }
}
