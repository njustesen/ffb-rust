//! 1:1 translation of `com.fumbbl.ffb.client.IProgressListener`.

/// Java: `com.fumbbl.ffb.client.IProgressListener`.
pub trait IProgressListener {
    fn init_progress(&mut self, minimum: i32, maximum: i32);
    fn update_progress(&mut self, progress: i32);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Recorder {
        minimum: i32,
        maximum: i32,
        progress: i32,
    }

    impl IProgressListener for Recorder {
        fn init_progress(&mut self, minimum: i32, maximum: i32) {
            self.minimum = minimum;
            self.maximum = maximum;
        }

        fn update_progress(&mut self, progress: i32) {
            self.progress = progress;
        }
    }

    #[test]
    fn init_progress_sets_bounds() {
        let mut r = Recorder::default();
        r.init_progress(0, 100);
        assert_eq!(r.minimum, 0);
        assert_eq!(r.maximum, 100);
    }

    #[test]
    fn update_progress_sets_progress() {
        let mut r = Recorder::default();
        r.update_progress(42);
        assert_eq!(r.progress, 42);
    }
}
