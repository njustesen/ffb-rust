/// 1:1 translation of com.fumbbl.ffb.client.model.OnlineAware (Java interface).
pub trait OnlineAware {
    fn set_online(&mut self, online: bool);
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Impl {
        online: bool,
    }
    impl OnlineAware for Impl {
        fn set_online(&mut self, online: bool) {
            self.online = online;
        }
    }

    #[test]
    fn set_online_true() {
        let mut i = Impl { online: false };
        i.set_online(true);
        assert!(i.online);
    }

    #[test]
    fn set_online_false() {
        let mut i = Impl { online: true };
        i.set_online(false);
        assert!(!i.online);
    }
}
