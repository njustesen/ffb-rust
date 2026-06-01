package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2016.MightyBlow;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class MightyBlowSkillTest {

    private MightyBlow skill;

    @BeforeEach
    void setUp() {
        skill = new MightyBlow();
        skill.postConstruct();
    }

    @Test
    void name_is_Mighty_Blow() {
        assertEquals("Mighty Blow", skill.getName());
    }

    @Test
    void category_is_strength() {
        assertEquals(SkillCategory.STRENGTH, skill.getCategory());
    }

    @Test
    void has_affects_either_armour_or_injury_on_block_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.affectsEitherArmourOrInjuryOnBlock));
    }

    @Test
    void is_bb2016_edition() {
        RulesCollection annotation = MightyBlow.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2016, annotation.value());
    }
}
