/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandInterceptorChoice`.
/// Sent when a coach selects an interceptor for a pass.
///
/// Note: `interceptionSkill` (a Skill object) is omitted — the Skill model type is not yet
/// directly accessible in the protocol crate. The interceptor player ID is the primary field.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandInterceptorChoice {
    /// Java: `fInterceptorId`
    pub interceptor_id: Option<String>,
    /// Java: `interceptionSkill` — skill ID string (simplified from full Skill object).
    pub interception_skill_id: Option<String>,
}

impl ClientCommandInterceptorChoice {
    pub fn new() -> Self { Self::default() }

    pub fn with_interceptor(interceptor_id: impl Into<String>) -> Self {
        Self {
            interceptor_id: Some(interceptor_id.into()),
            interception_skill_id: None,
        }
    }

    pub fn get_interceptor_id(&self) -> Option<&str> { self.interceptor_id.as_deref() }
    pub fn get_interception_skill_id(&self) -> Option<&str> { self.interception_skill_id.as_deref() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interceptor_id_stored() {
        let cmd = ClientCommandInterceptorChoice::with_interceptor("p5");
        assert_eq!(cmd.get_interceptor_id(), Some("p5"));
        assert!(cmd.get_interception_skill_id().is_none());
    }

    #[test]
    fn default_both_none() {
        let cmd = ClientCommandInterceptorChoice::new();
        assert!(cmd.interceptor_id.is_none());
    }
}
