package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2020.BreatheFire;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class BreatheFireSkillTest {

    private BreatheFire skill;

    @BeforeEach
    void setUp() {
        skill = new BreatheFire();
        skill.postConstruct();
    }

    @Test
    void name_is_Breathe_Fire() {
        assertEquals("Breathe Fire", skill.getName());
    }

    @Test
    void category_is_trait() {
        assertEquals(SkillCategory.TRAIT, skill.getCategory());
    }

    @Test
    void has_can_perform_armour_roll_instead_of_block_that_might_fail_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canPerformArmourRollInsteadOfBlockThatMightFailWithTurnover));
    }

    @Test
    void is_bb2020_edition() {
        RulesCollection annotation = BreatheFire.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2020, annotation.value());
    }
}
