package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2020.special.PumpUpTheCrowd;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class PumpUpTheCrowdSkillTest {

    private PumpUpTheCrowd skill;

    @BeforeEach
    void setUp() {
        skill = new PumpUpTheCrowd();
        skill.postConstruct();
    }

    @Test
    void name_is_Pump_Up_The_Crowd() {
        assertEquals("Pump Up The Crowd", skill.getName());
    }

    @Test
    void category_is_trait() {
        assertEquals(SkillCategory.TRAIT, skill.getCategory());
    }

    @Test
    void has_grants_team_re_roll_when_causing_cas_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.grantsTeamReRollWhenCausingCas));
    }

    @Test
    void is_bb2020_edition() {
        RulesCollection annotation = PumpUpTheCrowd.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2020, annotation.value());
    }
}
