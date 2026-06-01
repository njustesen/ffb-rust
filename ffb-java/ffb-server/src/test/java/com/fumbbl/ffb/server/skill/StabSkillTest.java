package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2016.Stab;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class StabSkillTest {

    private Stab skill;

    @BeforeEach
    void setUp() {
        skill = new Stab();
        skill.postConstruct();
    }

    @Test
    void name_is_Stab() {
        assertEquals("Stab", skill.getName());
    }

    @Test
    void category_is_extraordinary() {
        assertEquals(SkillCategory.EXTRAORDINARY, skill.getCategory());
    }

    @Test
    void has_can_perform_armour_roll_instead_of_block_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canPerformArmourRollInsteadOfBlock));
    }

    @Test
    void is_bb2016_edition() {
        RulesCollection annotation = Stab.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2016, annotation.value());
    }
}
