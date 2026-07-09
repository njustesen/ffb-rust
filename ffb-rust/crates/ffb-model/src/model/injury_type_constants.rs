/// 1:1 translation of com.fumbbl.ffb.model.InjuryTypeConstants.
pub const STUN: &str = "STUN";
pub const KO: &str = "KO";
pub const BH: &str = "BH";
pub const SI: &str = "SI";
pub const RIP: &str = "RIP";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stun_is_stun() {
        assert_eq!(STUN, "STUN");
    }

    #[test]
    fn rip_is_rip() {
        assert_eq!(RIP, "RIP");
    }
}
