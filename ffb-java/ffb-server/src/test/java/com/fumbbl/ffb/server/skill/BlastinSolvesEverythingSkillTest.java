package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2025.special.BlastinSolvesEverything;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class BlastinSolvesEverythingSkillTest {

    private BlastinSolvesEverything skill;

    @BeforeEach
    void setUp() {
        skill = new BlastinSolvesEverything();
        skill.postConstruct();
    }

    @Test
    void name_contains_blastin() {
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
    void is_bb2025_edition() {
        RulesCollection annotation = BlastinSolvesEverything.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2025, annotation.value());
    }
}
