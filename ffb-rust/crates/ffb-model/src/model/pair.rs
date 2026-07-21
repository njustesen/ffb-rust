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
