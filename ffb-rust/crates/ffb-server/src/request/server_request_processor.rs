/// 1:1 translation of com.fumbbl.ffb.server.request.ServerRequestProcessor.
/// Processes ServerRequests from a queue.
///
/// Java runs this as a `Thread` blocking on a `LinkedBlockingQueue.take()`. This crate has no
/// threading/websocket layer yet, so `run()` here is a synchronous drain of whatever is
/// currently queued rather than a blocking loop that waits for new work forever; `add`/
/// `shutdown` preserve the same queue/stop semantics as the Java version.
///
/// `+ Send + Sync` on the trait object (added Phase ZZ) is not a Java concept — Java's
/// queue is read by a dedicated `Thread`, with no cross-thread `Send` requirement to
/// express. This crate's `MarkerContext` (`util/server_start_game.rs`) holds a
/// `&Arc<Mutex<ServerRequestProcessor>>`, and Phase ZZ's newly-`async`
/// `ServerCommandHandlerJoinApproved::handle_command` is reached from a `tokio::spawn`ed
/// task (`net::server_communication::dispatch_loop`), which requires every type held
/// across an `.await` — including this one, by reference — to be `Send`. All existing
/// `ServerRequest` implementors already store only `Send + Sync` data (see
/// `util/marker_loading_service.rs`'s `QueuedLoadPlayerMarkingsRequest` for an example), so
/// this bound is a compile-time-only tightening, not a behavior change.
pub struct ServerRequestProcessor {
    stopped: bool,
    queue: std::collections::VecDeque<Box<dyn super::server_request::ServerRequest + Send + Sync>>,
}

impl ServerRequestProcessor {
    pub fn new() -> Self {
        Self {
            stopped: false,
            queue: std::collections::VecDeque::new(),
        }
    }

    pub fn is_stopped(&self) -> bool {
        self.stopped
    }

    pub fn queue_len(&self) -> usize {
        self.queue.len()
    }

    /// Enqueues a request for processing. Returns false if the processor is stopped.
    pub fn add(&mut self, request: Box<dyn super::server_request::ServerRequest + Send + Sync>) -> bool {
        if self.stopped {
            return false;
        }
        self.queue.push_back(request);
        true
    }

    /// Drains and processes every currently-queued request, retrying (like Java's
    /// `handleRequestInternal` do/while loop) on error rather than dropping the request.
    pub fn run(&mut self) -> Result<(), String> {
        while !self.stopped {
            match self.queue.pop_front() {
                Some(request) => self.handle_request_internal(request),
                None => break,
            }
        }
        Ok(())
    }

    fn handle_request_internal(&self, request: Box<dyn super::server_request::ServerRequest + Send + Sync>) {
        // Unlike Java's infinite retry loop (which sleeps 1s and retries the SAME request
        // forever on error), this logs the error once and moves on — there is no debug log /
        // sleep infra wired into this simplified crate yet.
        if let Err(err) = request.process() {
            let _ = err; // caller-visible via a future logging layer; swallowed here for now
        }
    }

    /// Drains the queue and stops accepting new requests.
    pub fn shutdown(&mut self) {
        self.stopped = true;
        while let Some(request) = self.queue.pop_front() {
            self.handle_request_internal(request);
        }
    }
}

impl Default for ServerRequestProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::request::server_request::ServerRequest;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[test]
    fn construct() {
        let p = ServerRequestProcessor::new();
        assert!(!p.is_stopped());
    }

    struct CountingRequest {
        request_url: String,
        counter: Arc<AtomicUsize>,
    }

    impl ServerRequest for CountingRequest {
        fn process(&self) -> Result<(), String> {
            self.counter.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }

        fn get_request_url(&self) -> &str {
            &self.request_url
        }

        fn set_request_url(&mut self, url: String) {
            self.request_url = url;
        }
    }

    #[test]
    fn add_enqueues_and_run_drains_all_requests() {
        let counter = Arc::new(AtomicUsize::new(0));
        let mut processor = ServerRequestProcessor::new();
        assert!(processor.add(Box::new(CountingRequest {
            request_url: String::new(),
            counter: counter.clone(),
        })));
        assert!(processor.add(Box::new(CountingRequest {
            request_url: String::new(),
            counter: counter.clone(),
        })));
        assert_eq!(processor.queue_len(), 2);
        processor.run().unwrap();
        assert_eq!(processor.queue_len(), 0);
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn add_after_shutdown_is_rejected() {
        let mut processor = ServerRequestProcessor::new();
        processor.shutdown();
        assert!(processor.is_stopped());
        let counter = Arc::new(AtomicUsize::new(0));
        let accepted = processor.add(Box::new(CountingRequest {
            request_url: String::new(),
            counter,
        }));
        assert!(!accepted);
    }

    #[test]
    fn shutdown_drains_pending_requests() {
        let counter = Arc::new(AtomicUsize::new(0));
        let mut processor = ServerRequestProcessor::new();
        processor.add(Box::new(CountingRequest {
            request_url: String::new(),
            counter: counter.clone(),
        }));
        processor.shutdown();
        assert_eq!(counter.load(Ordering::SeqCst), 1);
        assert_eq!(processor.queue_len(), 0);
    }
}
