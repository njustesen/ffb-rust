/// 1:1 translation of com.fumbbl.ffb.xml.IXmlReadable, adapted for Rust ownership.
///
/// Java: `IXmlReadable.startXmlElement(Game, String, Attributes) -> IXmlReadable` returns
/// either `this` (stay) or a newly-created child object to push onto the parse stack, and
/// that same child is often *also* stashed in a parent field (e.g. `Roster.fCurrentlyParsedRosterPosition`)
/// so the parent can read it back once the child's closing tag completes it. Rust can't alias
/// an owned value both ways, so `start_xml_element` returns `Some(child)` only to push (`None`
/// = stay), and `end_child` is the substitute for the "parent reads back its stashed field"
/// step: the driver hands the completed, popped child to the new stack top once its own
/// `end_xml_element` call returns `true`.
use std::any::Any;
use crate::model::game::Game;
use super::util_xml::XmlAttributes;

pub trait IXmlReadable {
    /// Java: `startXmlElement(Game, String, Attributes) -> IXmlReadable`.
    /// Returns `Some(child)` when a new element should be pushed onto the parse stack;
    /// `None` when this element stays on top (the child-vs-`this` comparison in Java).
    fn start_xml_element(
        &mut self,
        game: Option<&Game>,
        tag: &str,
        atts: &XmlAttributes,
    ) -> Option<Box<dyn IXmlReadable>>;

    /// Java: `endXmlElement(Game, String, String) -> boolean`. Returns `true` when this
    /// element's own closing tag was seen (element complete, pop continues upward).
    fn end_xml_element(&mut self, game: Option<&Game>, tag: &str, value: &str) -> bool;

    /// Rust-only substitute for Java's "parent reads back `fCurrentlyParsedXxx`" pattern —
    /// called with a child that just completed (its own `end_xml_element` returned `true`)
    /// so the new stack top (this element) can absorb it. Default no-op for leaf elements.
    fn end_child(&mut self, _tag: &str, _child: Box<dyn IXmlReadable>) {}

    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Consumes the box for a concrete-type downcast (`Box<dyn Any>::downcast`), needed by
    /// `end_child` implementations that take ownership of the completed child (e.g. `Roster`
    /// moving a completed `RosterPosition` into its position list).
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Leaf { seen_tag: Option<String> }

    impl IXmlReadable for Leaf {
        fn start_xml_element(&mut self, _game: Option<&Game>, _tag: &str, _atts: &XmlAttributes) -> Option<Box<dyn IXmlReadable>> {
            None
        }
        fn end_xml_element(&mut self, _game: Option<&Game>, tag: &str, _value: &str) -> bool {
            self.seen_tag = Some(tag.to_string());
            tag == "leaf"
        }
        fn as_any(&self) -> &dyn Any { self }
        fn as_any_mut(&mut self) -> &mut dyn Any { self }
        fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
    }

    #[test]
    fn start_xml_element_none_means_stay() {
        let mut leaf = Leaf { seen_tag: None };
        let atts = XmlAttributes::new();
        assert!(leaf.start_xml_element(None, "leaf", &atts).is_none());
    }

    #[test]
    fn end_xml_element_returns_true_when_own_tag_completes() {
        let mut leaf = Leaf { seen_tag: None };
        assert!(leaf.end_xml_element(None, "leaf", ""));
        assert!(!leaf.end_xml_element(None, "other", ""));
    }

    #[test]
    fn end_child_default_is_noop() {
        let mut leaf = Leaf { seen_tag: None };
        leaf.end_child("child", Box::new(Leaf { seen_tag: None }));
        // no panic, no state change expected
        assert!(leaf.seen_tag.is_none());
    }

    #[test]
    fn as_any_downcasts() {
        let leaf = Leaf { seen_tag: Some("x".into()) };
        let boxed: Box<dyn IXmlReadable> = Box::new(leaf);
        let downcast = boxed.as_any().downcast_ref::<Leaf>().unwrap();
        assert_eq!(downcast.seen_tag.as_deref(), Some("x"));
    }
}
