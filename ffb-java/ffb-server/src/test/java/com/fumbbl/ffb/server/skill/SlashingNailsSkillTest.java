package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2025.special.SlashingNails;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class SlashingNailsSkillTest {

    private SlashingNails skill;

    @BeforeEach
    void setUp() {
        skill = new SlashingNails();
        skill.postConstruct();
    }

    @Test
    void name_is_Slashing_Nails() {
        assertEquals("Slashing Nails", skill.getName());
    }

    @Test
    void category_is_trait() {
        assertEquals(SkillCategory.TRAIT, skill.getCategory());
    }

    @Test
    void has_can_gain_claws_for_blitz_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canGainClawsForBlitz));
    }

    @Test
    void is_bb2025_edition() {
        RulesCollection annotation = SlashingNails.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2025, annotation.value());
    }
}
