package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2025.special.WorkingInTandem;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class WorkingInTandemSkillTest {

    private WorkingInTandem skill;

    @BeforeEach
    void setUp() {
        skill = new WorkingInTandem();
        skill.postConstruct();
    }

    @Test
    void name_is_Working_in_Tandem() {
        assertEquals("Working in Tandem", skill.getName());
    }

    @Test
    void category_is_trait() {
        assertEquals(SkillCategory.TRAIT, skill.getCategory());
    }

    @Test
    void has_can_reroll_single_block_die_when_partner_is_marking_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canRerollSingleBlockDieWhenPartnerIsMarking));
    }

    @Test
    void is_bb2025_edition() {
        RulesCollection annotation = WorkingInTandem.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2025, annotation.value());
    }
}
