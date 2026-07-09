/// 1:1 translation of com.fumbbl.ffb.server.model.ISkillBehaviour (Java interface).
pub trait ISkillBehaviour {
    fn get_id(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Impl;
    impl ISkillBehaviour for Impl { fn get_id(&self) -> &str { "blockBehaviour" } }

    #[test]
    fn get_id_returns_id() {
        assert_eq!(Impl.get_id(), "blockBehaviour");
    }

    #[test]
    fn id_not_empty() {
        assert!(!Impl.get_id().is_empty());
    }
}
