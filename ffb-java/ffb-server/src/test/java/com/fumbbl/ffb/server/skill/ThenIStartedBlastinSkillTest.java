package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2020.special.ThenIStartedBlastin;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class ThenIStartedBlastinSkillTest {

    private ThenIStartedBlastin skill;

    @BeforeEach
    void setUp() {
        skill = new ThenIStartedBlastin();
        skill.postConstruct();
    }

    @Test
    void name_contains_Blastin() {
        assertTrue(skill.getName().contains("Blastin"));
    }

    @Test
    void category_is_trait() {
        assertEquals(SkillCategory.TRAIT, skill.getCategory());
    }

    @Test
    void has_can_blast_remote_player_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canBlastRemotePlayer));
    }

    @Test
    void is_bb2020_edition() {
        RulesCollection annotation = ThenIStartedBlastin.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2020, annotation.value());
    }
}
