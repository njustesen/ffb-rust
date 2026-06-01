package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2025.LethalFlight;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class LethalFlightSkillTest {

    private LethalFlight skill;

    @BeforeEach
    void setUp() {
        skill = new LethalFlight();
        skill.postConstruct();
    }

    @Test
    void name_is_Lethal_Flight() {
        assertEquals("Lethal Flight", skill.getName());
    }

    @Test
    void category_is_devious() {
        assertEquals(SkillCategory.DEVIOUS, skill.getCategory());
    }

    @Test
    void has_affects_either_armour_or_injury_on_ttm_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.affectsEitherArmourOrInjuryOnTtm));
    }

    @Test
    void is_bb2025_edition() {
        RulesCollection annotation = LethalFlight.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2025, annotation.value());
    }
}
