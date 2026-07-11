/// 1:1 translation of com.fumbbl.ffb.client.model.ControlAware (Java interface).
pub trait ControlAware {
    fn set_control(&mut self, control: bool);
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Impl {
        control: bool,
    }
    impl ControlAware for Impl {
        fn set_control(&mut self, control: bool) {
            self.control = control;
        }
    }

    #[test]
    fn set_control_true() {
        let mut i = Impl { control: false };
        i.set_control(true);
        assert!(i.control);
    }

    #[test]
    fn set_control_false() {
        let mut i = Impl { control: true };
        i.set_control(false);
        assert!(!i.control);
    }
}
