package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.common.StandFirm;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class StandFirmSkillTest {

    private StandFirm skill;

    @BeforeEach
    void setUp() {
        skill = new StandFirm();
        skill.postConstruct();
    }

    @Test
    void name_is_Stand_Firm() {
        assertEquals("Stand Firm", skill.getName());
    }

    @Test
    void category_is_strength() {
        assertEquals(SkillCategory.STRENGTH, skill.getCategory());
    }

    @Test
    void has_can_refuse_to_be_pushed_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canRefuseToBePushed));
    }

    @Test
    void is_common_across_all_editions() {
        RulesCollection annotation = StandFirm.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.COMMON, annotation.value());
    }
}
