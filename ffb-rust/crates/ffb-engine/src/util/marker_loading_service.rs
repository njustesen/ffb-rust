// 1:1 translation of com.fumbbl.ffb.server.util.MarkerLoadingService
//
// The Java source contains a single method `loadMarker` that dispatches to either
// a FUMBBL HTTP API request (FumbblRequestLoadPlayerMarkings) or a DB query
// (DbPlayerMarkersQuery). Both paths are server infrastructure with no pure
// calculation logic, so they are skipped per translation rules (skip DB/HTTP/file-loading).
//
// This file is a structural placeholder that preserves the class boundary.

pub struct MarkerLoadingService;

impl MarkerLoadingService {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MarkerLoadingService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn marker_loading_service_default_constructs() {
        let _svc = MarkerLoadingService::default();
    }

    #[test]
    fn marker_loading_service_new_constructs() {
        let _svc = MarkerLoadingService::new();
    }

    // The Java loadMarker method dispatches exclusively to DB/HTTP infrastructure
    // (DbPlayerMarkersQuery and FumbblRequestLoadPlayerMarkings). These are
    // server-side I/O operations with no pure calculation to extract or test.
    // Per translation rules, DB/HTTP/file-loading methods are skipped.
}
