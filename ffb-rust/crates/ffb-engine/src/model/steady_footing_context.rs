/// Re-export of SteadyFootingContext, translated from
/// com.fumbbl.ffb.server.model.SteadyFootingContext.
pub use crate::drop_player_context::{SteadyFootingContext, SteadyFootingInner};

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::ApothecaryMode;
    use crate::drop_player_context::DropPlayerContext;
    use crate::injury::InjuryResult;

    #[test]
    fn from_drop_player_context() {
        let ctx = DropPlayerContext::new();
        let sfc = SteadyFootingContext::from_drop_player(ctx);
        assert!(sfc.drop_player_context().is_some());
        assert!(sfc.injury_result().is_none());
        assert!(sfc.injury_type_name().is_none());
    }

    #[test]
    fn from_injury_result() {
        let ir = InjuryResult::new(ApothecaryMode::Defender);
        let sfc = SteadyFootingContext::from_injury_result(ir);
        assert!(sfc.injury_result().is_some());
        assert!(sfc.drop_player_context().is_none());
    }

    #[test]
    fn from_injury_type_name() {
        let sfc = SteadyFootingContext::from_injury_type_name("InjuryTypeBlock".to_string());
        assert_eq!(sfc.injury_type_name(), Some("InjuryTypeBlock"));
        assert!(sfc.drop_player_context().is_none());
        assert!(sfc.injury_result().is_none());
    }

    #[test]
    fn get_apothecary_mode_from_drop_player_context() {
        let mut ctx = DropPlayerContext::new();
        ctx.apothecary_mode = Some(ApothecaryMode::Defender);
        let sfc = SteadyFootingContext::from_drop_player(ctx);
        assert_eq!(sfc.get_apothecary_mode(), ApothecaryMode::Defender);
    }

    #[test]
    fn get_apothecary_mode_default_for_injury_type_name() {
        let sfc = SteadyFootingContext::from_injury_type_name("SomeType".to_string());
        assert_eq!(sfc.get_apothecary_mode(), ApothecaryMode::Attacker);
    }

    #[test]
    fn get_apothecary_mode_from_injury_result() {
        let ir = InjuryResult::new(ApothecaryMode::Attacker);
        let sfc = SteadyFootingContext::from_injury_result(ir);
        assert_eq!(sfc.get_apothecary_mode(), ApothecaryMode::Attacker);
    }

    #[test]
    fn from_injury_type_name_injury_result_is_none() {
        let sfc = SteadyFootingContext::from_injury_type_name("Block".to_string());
        assert!(sfc.injury_result().is_none());
    }

    #[test]
    fn from_drop_player_injury_type_name_is_none() {
        let ctx = DropPlayerContext::new();
        let sfc = SteadyFootingContext::from_drop_player(ctx);
        assert!(sfc.injury_type_name().is_none());
    }
}
