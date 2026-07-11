/// 1:1 translation of com.fumbbl.ffb.xml.XmlHandler, driven by `quick_xml::Reader` instead
/// of a SAX `XMLReader` (quick-xml's `Event::Start`/`Text`/`End`/`Empty` map onto SAX's
/// `startElement`/`characters`/`endElement` one-to-one; `Empty` is SAX's start+end pair
/// collapsed into a single self-closing-tag event, handled here by driving both steps).
use quick_xml::events::Event;
use quick_xml::Reader;

use crate::model::game::Game;
use super::i_xml_readable::IXmlReadable;
use super::util_xml::XmlAttributes;

pub struct XmlHandler {
    value: String,
    stack: Vec<Box<dyn IXmlReadable>>,
}

impl XmlHandler {
    /// Java: `new XmlHandler(Game, IXmlReadable)`.
    pub fn new(parsed_element: Box<dyn IXmlReadable>) -> Self {
        XmlHandler { value: String::new(), stack: vec![parsed_element] }
    }

    /// Java: `characters(char[], int, int)`.
    fn characters(&mut self, text: &str) {
        self.value.push_str(text);
    }

    /// Java: `startElement(String, String, String, Attributes)`.
    fn start_element(&mut self, game: Option<&Game>, tag: &str, atts: &XmlAttributes) {
        if let Some(current) = self.stack.last_mut() {
            if let Some(child) = current.start_xml_element(game, tag, atts) {
                self.stack.push(child);
            }
        }
    }

    /// Java: `endElement(String, String, String)`.
    ///
    /// Java keeps a separate `fParsedElement` reference alongside the stack, so even
    /// though the root object is eventually popped off the stack when its own closing
    /// tag completes, `getParsedElement()` still returns it (same object, untouched by
    /// the stack manipulation). Rust can't alias one owned box both ways, so this pins
    /// the bottommost (root) stack entry — it is consulted but never popped — which
    /// yields the same final object state without needing `Rc<RefCell<_>>`.
    fn end_element(&mut self, game: Option<&Game>, tag: &str) {
        let value = self.value.trim().to_string();
        let mut completed_child: Option<Box<dyn IXmlReadable>> = None;
        loop {
            if self.stack.len() <= 1 {
                if let Some(root) = self.stack.last_mut() {
                    if let Some(child) = completed_child.take() {
                        root.end_child(tag, child);
                    }
                    root.end_xml_element(game, tag, &value);
                }
                break;
            }
            let mut current = self.stack.pop().unwrap();
            if let Some(child) = completed_child.take() {
                current.end_child(tag, child);
            }
            let complete = current.end_xml_element(game, tag, &value);
            if !complete {
                self.stack.push(current);
                break;
            }
            completed_child = Some(current);
        }
        self.value.clear();
    }

    /// Java: `XmlHandler.parse(Game, InputSource, IXmlReadable)`. Consumes the handler
    /// (built via `new`) and returns the fully-parsed root element.
    pub fn parse(game: Option<&Game>, xml: &str, parsed_element: Box<dyn IXmlReadable>) -> Box<dyn IXmlReadable> {
        let mut handler = XmlHandler::new(parsed_element);
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(false);

        loop {
            match reader.read_event() {
                Ok(Event::Start(e)) => {
                    let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    let atts = collect_attributes(&e);
                    handler.start_element(game, &tag, &atts);
                }
                Ok(Event::Empty(e)) => {
                    let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    let atts = collect_attributes(&e);
                    handler.start_element(game, &tag, &atts);
                    handler.end_element(game, &tag);
                }
                Ok(Event::Text(e)) => {
                    let text = e.unescape().unwrap_or_default();
                    handler.characters(&text);
                }
                Ok(Event::End(e)) => {
                    let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    handler.end_element(game, &tag);
                }
                Ok(Event::Eof) => break,
                Ok(_) => {}
                Err(e) => panic!("XML parsing error: {e}"),
            }
        }

        handler.stack.into_iter().next().expect("parse stack must retain the root element")
    }
}

fn collect_attributes(e: &quick_xml::events::BytesStart) -> XmlAttributes {
    let mut atts = XmlAttributes::new();
    for attr in e.attributes().flatten() {
        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
        let value = attr.unescape_value().unwrap_or_default().to_string();
        atts.insert(key, value);
    }
    atts
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::Any;

    /// Synthetic parent/child pair exercising the multi-pop-on-close loop: `<root>` holds
    /// a `<child>` element; on `</child>` the parent must receive it via `end_child`.
    #[derive(Default)]
    struct TestChild { captured_value: String }

    impl IXmlReadable for TestChild {
        fn start_xml_element(&mut self, _game: Option<&Game>, _tag: &str, _atts: &XmlAttributes) -> Option<Box<dyn IXmlReadable>> {
            None
        }
        fn end_xml_element(&mut self, _game: Option<&Game>, tag: &str, value: &str) -> bool {
            if tag == "value" {
                self.captured_value = value.to_string();
            }
            tag == "child"
        }
        fn as_any(&self) -> &dyn Any { self }
        fn as_any_mut(&mut self) -> &mut dyn Any { self }
        fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
    }

    #[derive(Default)]
    struct TestRoot { child_value: Option<String> }

    impl IXmlReadable for TestRoot {
        fn start_xml_element(&mut self, _game: Option<&Game>, tag: &str, _atts: &XmlAttributes) -> Option<Box<dyn IXmlReadable>> {
            if tag == "child" {
                Some(Box::new(TestChild::default()))
            } else {
                None
            }
        }
        fn end_xml_element(&mut self, _game: Option<&Game>, tag: &str, _value: &str) -> bool {
            tag == "root"
        }
        fn end_child(&mut self, tag: &str, child: Box<dyn IXmlReadable>) {
            if tag == "child" {
                let c = child.as_any().downcast_ref::<TestChild>().unwrap();
                self.child_value = Some(c.captured_value.clone());
            }
        }
        fn as_any(&self) -> &dyn Any { self }
        fn as_any_mut(&mut self) -> &mut dyn Any { self }
        fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
    }

    #[test]
    fn parent_captures_completed_child_via_end_child() {
        let xml = "<root><child><value>hello</value></child></root>";
        let parsed = XmlHandler::parse(None, xml, Box::new(TestRoot::default()));
        let root = parsed.as_any().downcast_ref::<TestRoot>().unwrap();
        assert_eq!(root.child_value.as_deref(), Some("hello"));
    }

    #[test]
    fn characters_fragmented_across_entities_accumulates() {
        // The "&amp;" entity forces quick-xml to split text into multiple Text events
        // around it; XmlHandler must still accumulate all of them before trimming.
        let xml = "<root><child><value>A &amp; B</value></child></root>";
        let parsed = XmlHandler::parse(None, xml, Box::new(TestRoot::default()));
        let root = parsed.as_any().downcast_ref::<TestRoot>().unwrap();
        assert_eq!(root.child_value.as_deref(), Some("A & B"));
    }

    #[test]
    fn whitespace_only_value_trims_to_empty() {
        let xml = "<root><child><value>   </value></child></root>";
        let parsed = XmlHandler::parse(None, xml, Box::new(TestRoot::default()));
        let root = parsed.as_any().downcast_ref::<TestRoot>().unwrap();
        assert_eq!(root.child_value.as_deref(), Some(""));
    }

    #[test]
    fn root_object_survives_its_own_closing_tag() {
        // The root's own </root> completes it (end_xml_element("root",...) returns true);
        // it must still be retrievable afterward, not dropped when the stack empties.
        let xml = "<root><child><value>x</value></child></root>";
        let parsed = XmlHandler::parse(None, xml, Box::new(TestRoot::default()));
        let root = parsed.as_any().downcast_ref::<TestRoot>();
        assert!(root.is_some());
    }

    #[test]
    fn stack_never_underflows_on_unbalanced_close() {
        // A root that always reports "not complete" must not panic when the loop
        // runs out of stack entries to pop.
        struct NeverComplete;
        impl IXmlReadable for NeverComplete {
            fn start_xml_element(&mut self, _g: Option<&Game>, _t: &str, _a: &XmlAttributes) -> Option<Box<dyn IXmlReadable>> { None }
            fn end_xml_element(&mut self, _g: Option<&Game>, _t: &str, _v: &str) -> bool { false }
            fn as_any(&self) -> &dyn Any { self }
            fn as_any_mut(&mut self) -> &mut dyn Any { self }
            fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
        }
        let xml = "<root></root>";
        let _ = XmlHandler::parse(None, xml, Box::new(NeverComplete));
    }
}
