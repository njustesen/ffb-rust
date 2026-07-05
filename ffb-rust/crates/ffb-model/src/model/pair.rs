/// 1:1 translation of com.fumbbl.ffb.Pair<L,R>.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pair<L, R> {
    pub left: L,
    pub right: R,
}

impl<L, R> Pair<L, R> {
    pub fn new(left: L, right: R) -> Self {
        Pair { left, right }
    }

    pub fn get_left(&self) -> &L { &self.left }
    pub fn get_right(&self) -> &R { &self.right }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn new_stores_left_and_right() {
        let p = Pair::new(1, "hello");
        assert_eq!(p.get_left(), &1);
        assert_eq!(p.get_right(), &"hello");
    }
    #[test]
    fn equality_works() {
        assert_eq!(Pair::new(1, 2), Pair::new(1, 2));
        assert_ne!(Pair::new(1, 2), Pair::new(1, 3));
    }
}
