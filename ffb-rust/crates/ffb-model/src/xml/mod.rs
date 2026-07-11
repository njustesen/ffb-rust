/// 1:1 translation of the read-side of com.fumbbl.ffb.xml (the standalone-server's
/// disk roster/team XML pipeline). The write side (`IXmlSerializable`/`addToXml`) is
/// out of scope — nothing in this crate needs to produce this XML format.
pub mod i_xml_readable;
pub mod util_xml;
pub mod xml_handler;

pub use i_xml_readable::IXmlReadable;
pub use util_xml::XmlAttributes;
pub use xml_handler::XmlHandler;
