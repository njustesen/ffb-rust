package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2020.ProjectileVomit;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class ProjectileVomitSkillTest {

    private ProjectileVomit skill;

    @BeforeEach
    void setUp() {
        skill = new ProjectileVomit();
        skill.postConstruct();
    }

    @Test
    void name_is_Projectile_Vomit() {
        assertEquals("Projectile Vomit", skill.getName());
    }

    @Test
    void category_is_trait() {
        assertEquals(SkillCategory.TRAIT, skill.getCategory());
    }

    @Test
    void has_provides_block_alternative_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.providesBlockAlternative));
    }

    @Test
    void is_bb2020_edition() {
        RulesCollection annotation = ProjectileVomit.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2020, annotation.value());
    }
}
