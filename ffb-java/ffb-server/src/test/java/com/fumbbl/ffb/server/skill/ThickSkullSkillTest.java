package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.common.ThickSkull;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class ThickSkullSkillTest {

    private ThickSkull skill;

    @BeforeEach
    void setUp() {
        skill = new ThickSkull();
        skill.postConstruct();
    }

    @Test
    void name_is_Thick_Skull() {
        assertEquals("Thick Skull", skill.getName());
    }

    @Test
    void category_is_strength() {
        assertEquals(SkillCategory.STRENGTH, skill.getCategory());
    }

    @Test
    void has_convert_ko_to_stun_on_8_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.convertKOToStunOn8));
    }

    @Test
    void is_common_across_all_editions() {
        RulesCollection annotation = ThickSkull.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.COMMON, annotation.value());
    }
}
