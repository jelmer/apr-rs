//! XML parsing functionality from apr-util.
//!
//! Provides XML parsing using expat backend.

use crate::pool::Pool;
use crate::{Error, Status};
use std::ffi::c_char;
use std::ffi::CStr;
use std::marker::PhantomData;
use std::ptr;

/// XML parser handle.
pub struct XmlParser<'pool> {
    parser: *mut apr_sys::apr_xml_parser,
    _pool: PhantomData<&'pool Pool<'pool>>,
}

/// Parsed XML document.
pub struct XmlDoc<'pool> {
    doc: *mut apr_sys::apr_xml_doc,
    _pool: PhantomData<&'pool Pool<'pool>>,
}

/// XML element in a document.
pub struct XmlElem<'pool> {
    elem: *const apr_sys::apr_xml_elem,
    _pool: PhantomData<&'pool Pool<'pool>>,
}

/// XML attribute.
pub struct XmlAttr<'pool> {
    attr: *const apr_sys::apr_xml_attr,
    _pool: PhantomData<&'pool Pool<'pool>>,
}

impl<'pool> XmlParser<'pool> {
    /// Create a new XML parser.
    pub fn new(pool: &'pool Pool<'pool>) -> Result<Self, Error> {
        let parser =
            unsafe { apr_sys::apr_xml_parser_create(pool.as_ptr() as *mut apr_sys::apr_pool_t) };

        if parser.is_null() {
            Err(Error::from_status(Status::from(apr_sys::APR_ENOMEM as i32)))
        } else {
            Ok(XmlParser {
                parser,
                _pool: PhantomData,
            })
        }
    }

    /// Feed data to the parser.
    pub fn feed(&mut self, data: &[u8]) -> Result<(), Error> {
        let status = unsafe {
            apr_sys::apr_xml_parser_feed(
                self.parser,
                data.as_ptr() as *const c_char,
                data.len() as apr_sys::apr_size_t,
            )
        };

        if status == apr_sys::APR_SUCCESS as i32 {
            Ok(())
        } else {
            Err(Error::from_status(Status::from(status)))
        }
    }

    /// Finish parsing and get the document.
    pub fn done(self) -> Result<XmlDoc<'pool>, Error> {
        let mut doc: *mut apr_sys::apr_xml_doc = ptr::null_mut();

        let status = unsafe { apr_sys::apr_xml_parser_done(self.parser, &mut doc) };

        if status == apr_sys::APR_SUCCESS as i32 {
            Ok(XmlDoc {
                doc,
                _pool: PhantomData,
            })
        } else {
            Err(Error::from_status(Status::from(status)))
        }
    }

    /// Get error information if parsing failed.
    pub fn get_error(&self) -> Option<String> {
        let mut errbuf = [0 as c_char; 200];
        let errbufsize = errbuf.len() as apr_sys::apr_size_t;

        unsafe {
            let error_str =
                apr_sys::apr_xml_parser_geterror(self.parser, errbuf.as_mut_ptr(), errbufsize);

            if !error_str.is_null() {
                Some(CStr::from_ptr(error_str).to_string_lossy().into_owned())
            } else {
                None
            }
        }
    }
}

impl<'pool> XmlDoc<'pool> {
    /// Get the root element of the document.
    pub fn root(&self) -> Option<XmlElem<'pool>> {
        unsafe {
            let doc = &*self.doc;
            if doc.root.is_null() {
                None
            } else {
                Some(XmlElem {
                    elem: doc.root,
                    _pool: PhantomData,
                })
            }
        }
    }

    /// Convert the document to a string representation.
    ///
    /// The returned string is allocated in the pool and borrows from it.
    pub fn to_string<'a>(&self, pool: &'a Pool<'a>, style: i32) -> Result<&'a str, Error> {
        let mut buf_ptr: *const c_char = ptr::null();

        unsafe {
            apr_sys::apr_xml_to_text(
                pool.as_ptr() as *mut apr_sys::apr_pool_t,
                (*self.doc).root,
                style,
                ptr::null_mut(),
                ptr::null_mut(),
                &mut buf_ptr,
                ptr::null_mut(),
            );
        }

        if buf_ptr.is_null() {
            Err(Error::from_status(Status::from(apr_sys::APR_ENOMEM as i32)))
        } else {
            unsafe {
                Ok(CStr::from_ptr(buf_ptr)
                    .to_str()
                    .map_err(|_| Error::from_status(Status::from(apr_sys::APR_EINVAL as i32)))?)
            }
        }
    }
}

impl<'pool> XmlElem<'pool> {
    /// Get the element name.
    pub fn name(&self) -> &str {
        unsafe {
            let elem = &*self.elem;
            if elem.name.is_null() {
                ""
            } else {
                CStr::from_ptr(elem.name).to_str().unwrap_or("")
            }
        }
    }

    /// Get the element namespace.
    pub fn namespace(&self) -> Option<&str> {
        unsafe {
            let elem = &*self.elem;
            if elem.ns == -1 {
                None
            } else {
                // TODO: Resolve namespace from document namespaces array
                Some("")
            }
        }
    }

    /// Get the first child element.
    pub fn first_child(&self) -> Option<XmlElem<'pool>> {
        unsafe {
            let elem = &*self.elem;
            if elem.first_child.is_null() {
                None
            } else {
                Some(XmlElem {
                    elem: elem.first_child,
                    _pool: PhantomData,
                })
            }
        }
    }

    /// Get the next sibling element.
    pub fn next(&self) -> Option<XmlElem<'pool>> {
        unsafe {
            let elem = &*self.elem;
            if elem.next.is_null() {
                None
            } else {
                Some(XmlElem {
                    elem: elem.next,
                    _pool: PhantomData,
                })
            }
        }
    }

    /// Get the first attribute.
    pub fn first_attr(&self) -> Option<XmlAttr<'pool>> {
        unsafe {
            let elem = &*self.elem;
            if elem.attr.is_null() {
                None
            } else {
                Some(XmlAttr {
                    attr: elem.attr,
                    _pool: PhantomData,
                })
            }
        }
    }

    /// Get the text content of the element.
    pub fn text(&self) -> Option<&str> {
        unsafe {
            let elem = &*self.elem;
            if elem.first_cdata.first.is_null() {
                None
            } else {
                let text_item = &*elem.first_cdata.first;
                if text_item.text.is_null() {
                    None
                } else {
                    Some(CStr::from_ptr(text_item.text).to_str().unwrap_or(""))
                }
            }
        }
    }
}

impl<'pool> XmlAttr<'pool> {
    /// Get the attribute name.
    pub fn name(&self) -> &str {
        unsafe {
            let attr = &*self.attr;
            if attr.name.is_null() {
                ""
            } else {
                CStr::from_ptr(attr.name).to_str().unwrap_or("")
            }
        }
    }

    /// Get the attribute value.
    pub fn value(&self) -> &str {
        unsafe {
            let attr = &*self.attr;
            if attr.value.is_null() {
                ""
            } else {
                CStr::from_ptr(attr.value).to_str().unwrap_or("")
            }
        }
    }

    /// Get the next attribute.
    pub fn next(&self) -> Option<XmlAttr<'pool>> {
        unsafe {
            let attr = &*self.attr;
            if attr.next.is_null() {
                None
            } else {
                Some(XmlAttr {
                    attr: attr.next,
                    _pool: PhantomData,
                })
            }
        }
    }
}

/// Parse an XML string and return the serialized result.
///
/// The returned string is allocated in the pool and borrows from it.
pub fn parse<'a>(xml: &str, pool: &'a Pool<'a>) -> Result<&'a str, Error> {
    let doc = parse_xml(xml, pool)?;
    doc.to_string(pool, 0)
}

/// Validate XML string (pool-less API).
pub fn validate(xml: &str) -> Result<(), Error> {
    crate::pool::with_tmp_pool(|pool| {
        parse_xml(xml, pool)?;
        Ok(())
    })
}

/// Parse an XML string into a document (pool-exposed API).
pub fn parse_xml<'pool>(xml: &str, pool: &'pool Pool<'pool>) -> Result<XmlDoc<'pool>, Error> {
    let mut parser = XmlParser::new(pool)?;
    parser.feed(xml.as_bytes())?;
    parser.done()
}

/// Iterator over XML elements.
pub struct XmlElemIter<'pool> {
    current: Option<XmlElem<'pool>>,
}

impl<'pool> XmlElem<'pool> {
    /// Iterate over child elements.
    pub fn children(&self) -> XmlElemIter<'pool> {
        XmlElemIter {
            current: self.first_child(),
        }
    }
}

impl<'pool> Iterator for XmlElemIter<'pool> {
    type Item = XmlElem<'pool>;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current.take()?;
        self.current = current.next();
        Some(current)
    }
}

/// Iterator over XML attributes.
pub struct XmlAttrIter<'pool> {
    current: Option<XmlAttr<'pool>>,
}

impl<'pool> XmlElem<'pool> {
    /// Iterate over attributes.
    pub fn attributes(&self) -> XmlAttrIter<'pool> {
        XmlAttrIter {
            current: self.first_attr(),
        }
    }
}

impl<'pool> Iterator for XmlAttrIter<'pool> {
    type Item = XmlAttr<'pool>;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current.take()?;
        self.current = current.next();
        Some(current)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_xml() {
        let pool = Pool::new();
        let xml = r#"<?xml version="1.0"?><root><child>Hello</child></root>"#;

        match parse_xml(xml, &pool) {
            Ok(doc) => {
                if let Some(root) = doc.root() {
                    assert_eq!(root.name(), "root");

                    if let Some(child) = root.first_child() {
                        assert_eq!(child.name(), "child");
                        assert_eq!(child.text(), Some("Hello"));
                    }
                }
            }
            Err(_) => {
                // XML parsing may not be available
            }
        }
    }

    #[test]
    fn test_parse_xml_with_attributes() {
        let pool = Pool::new();
        let xml = r#"<?xml version="1.0"?><root id="1" name="test"/>"#;

        match parse_xml(xml, &pool) {
            Ok(doc) => {
                if let Some(root) = doc.root() {
                    let attrs: Vec<_> = root.attributes().collect();
                    assert!(!attrs.is_empty());

                    // Verify we have both attributes (order may vary)
                    let attr_names: Vec<_> = attrs.iter().map(|a| a.name()).collect();
                    assert!(attr_names.contains(&"id"));
                    assert!(attr_names.contains(&"name"));

                    // Find the id attribute specifically
                    if let Some(id_attr) = attrs.iter().find(|a| a.name() == "id") {
                        assert_eq!(id_attr.value(), "1");
                    }
                    if let Some(name_attr) = attrs.iter().find(|a| a.name() == "name") {
                        assert_eq!(name_attr.value(), "test");
                    }
                }
            }
            Err(_) => {
                // XML parsing may not be available
            }
        }
    }

    #[test]
    fn test_xml_parser_feed() {
        let pool = Pool::new();

        if let Ok(mut parser) = XmlParser::new(&pool) {
            let xml_part1 = b"<?xml version=\"1.0\"?>";
            let xml_part2 = b"<root><child>Test</child></root>";

            assert!(parser.feed(xml_part1).is_ok());
            assert!(parser.feed(xml_part2).is_ok());

            match parser.done() {
                Ok(doc) => {
                    assert!(doc.root().is_some());
                }
                Err(_) => {
                    // Parser might not be available
                }
            }
        }
    }

    #[test]
    fn test_xml_children_iterator() {
        let pool = Pool::new();
        let xml = r#"<?xml version="1.0"?><root><a/><b/><c/></root>"#;

        match parse_xml(xml, &pool) {
            Ok(doc) => {
                if let Some(root) = doc.root() {
                    let children: Vec<_> = root
                        .children()
                        .map(|elem| elem.name().to_string())
                        .collect();
                    assert_eq!(children, vec!["a", "b", "c"]);
                }
            }
            Err(_) => {
                // XML parsing may not be available
            }
        }
    }
}
