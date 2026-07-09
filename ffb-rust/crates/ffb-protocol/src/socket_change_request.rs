/// 1:1 translation of `com.fumbbl.ffb.net.SocketChangeRequest`.
/// Carries a pending NIO selector operation (register or change-ops) for a
/// socket channel.  The Java version holds an actual `SocketChannel`; in Rust
/// we identify the socket by a plain integer handle so the struct stays
/// `Send + Sync` without unsafe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SocketChangeRequest {
    /// Opaque handle that identifies the socket channel.
    pub socket_handle: usize,
    /// Operation type: `REGISTER` (1) or `CHANGEOPS` (2).
    pub request_type: i32,
    /// The NIO `SelectionKey` ops bitmask (e.g. `OP_READ`, `OP_WRITE`).
    pub ops: i32,
}

impl SocketChangeRequest {
    /// `REGISTER` operation constant (Java: `static final int REGISTER = 1`).
    pub const REGISTER: i32 = 1;
    /// `CHANGEOPS` operation constant (Java: `static final int CHANGEOPS = 2`).
    pub const CHANGEOPS: i32 = 2;

    pub fn new(socket_handle: usize, request_type: i32, ops: i32) -> Self {
        Self { socket_handle, request_type, ops }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_match_java() {
        assert_eq!(SocketChangeRequest::REGISTER, 1);
        assert_eq!(SocketChangeRequest::CHANGEOPS, 2);
    }

    #[test]
    fn new_sets_fields() {
        let req = SocketChangeRequest::new(42, SocketChangeRequest::REGISTER, 1);
        assert_eq!(req.socket_handle, 42);
        assert_eq!(req.request_type, SocketChangeRequest::REGISTER);
        assert_eq!(req.ops, 1);
    }

    #[test]
    fn equality() {
        let a = SocketChangeRequest::new(1, 1, 4);
        let b = SocketChangeRequest::new(1, 1, 4);
        assert_eq!(a, b);
    }
}
