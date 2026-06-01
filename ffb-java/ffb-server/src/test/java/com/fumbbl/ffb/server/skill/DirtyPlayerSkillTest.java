package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2016.DirtyPlayer;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class DirtyPlayerSkillTest {

    private DirtyPlayer skill;

    @BeforeEach
    void setUp() {
        skill = new DirtyPlayer();
        skill.postConstruct();
    }

    @Test
    void name_is_Dirty_Player() {
        assertEquals("Dirty Player", skill.getName());
    }

    @Test
    void category_is_general() {
        assertEquals(SkillCategory.GENERAL, skill.getCategory());
    }

    @Test
    void has_affectsEitherArmourOrInjuryOnFoul_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.affectsEitherArmourOrInjuryOnFoul),
            "Dirty Player must register affectsEitherArmourOrInjuryOnFoul so only one of armour or injury is boosted");
    }

    @Test
    void does_not_have_forceFollowup_property() {
        assertFalse(skill.hasSkillProperty(NamedProperties.forceFollowup),
            "Dirty Player does not force follow-up");
    }

    @Test
    void is_bb2016_edition() {
        RulesCollection annotation = DirtyPlayer.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation, "DirtyPlayer must have a RulesCollection annotation");
        assertEquals(RulesCollection.Rules.BB2016, annotation.value());
    }
}
