use serde::{Deserialize, Serialize};

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogWithoutParameter.
/// Abstract base for dialogs that carry no data beyond their ID.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogWithoutParameter;
