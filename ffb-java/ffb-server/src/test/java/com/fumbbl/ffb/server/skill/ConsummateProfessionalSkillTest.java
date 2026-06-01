package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2020.special.ConsummateProfessional;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class ConsummateProfessionalSkillTest {

    private ConsummateProfessional skill;

    @BeforeEach
    void setUp() {
        skill = new ConsummateProfessional();
        skill.postConstruct();
    }

    @Test
    void name_is_Consummate_Professional() {
        assertEquals("Consummate Professional", skill.getName());
    }

    @Test
    void category_is_trait() {
        assertEquals(SkillCategory.TRAIT, skill.getCategory());
    }

    @Test
    void has_can_reroll_single_die_once_per_period_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canRerollSingleDieOncePerPeriod));
    }

    @Test
    void is_bb2020_edition() {
        RulesCollection annotation = ConsummateProfessional.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2020, annotation.value());
    }
}
