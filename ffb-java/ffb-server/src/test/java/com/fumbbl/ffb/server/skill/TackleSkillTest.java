package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.CancelSkillProperty;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.common.Tackle;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class TackleSkillTest {

    private Tackle skill;

    @BeforeEach
    void setUp() {
        skill = new Tackle();
        skill.postConstruct();
    }

    @Test
    void name_is_Tackle() {
        assertEquals("Tackle", skill.getName());
    }

    @Test
    void category_is_general() {
        assertEquals(SkillCategory.GENERAL, skill.getCategory());
    }

    @Test
    void has_canRerollDodge_cancel_property() {
        assertTrue(skill.hasSkillProperty(new CancelSkillProperty(NamedProperties.canRerollDodge)),
            "Tackle must cancel the opponent's Dodge re-roll");
    }

    @Test
    void does_not_have_forceFollowup_property() {
        assertFalse(skill.hasSkillProperty(NamedProperties.forceFollowup),
            "Tackle does not force the attacker to follow up");
    }

    @Test
    void is_common_across_all_editions() {
        RulesCollection annotation = Tackle.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation, "Tackle must have a RulesCollection annotation");
        assertEquals(RulesCollection.Rules.COMMON, annotation.value());
    }
}
