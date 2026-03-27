//!  Build a DOM-like Document Tree
//!
//!  Build a simplified DOM tree that models an HTML-like document structure, exercising all smart pointer concepts together.
//!
//!  1. Node struct
//!
//!  Each node has: tag (String), attributes (key-value map), children, parent, and optional text. Choose the correct smart pointer for each field.
//!
//!  2. TextContent with Deref
//!
//!  A newtype wrapping String. Implement Deref<Target = str> so it can be passed to any fn(&str).
//!
//!  3. Tree API
//!
//!  - Node::new(tag) -> Rc<RefCell<Node>>
//!  - Node::add_child(parent, child) — appends child, sets parent back-reference
//!  - Node::set_attribute(&mut self, key, value)
//!  - Node::set_text(&mut self, text)
//!  - Node::remove_child(parent, child) — removes child, clears parent ref
//!
//!  4. Query methods
//!
//!  - parent(&self) -> Option<Rc<RefCell<Node>>> — upgrade from Weak
//!  - depth(node) -> usize — root = 0
//!  - find_by_tag(node, tag) -> Vec<Rc<RefCell<Node>>> — recursive search
//!
//!  5. render method
//!
//!  Produce indented HTML:
//!  <html>
//!    <head>
//!      <title>Hello</title>
//!    </head>
//!    <body>
//!      <div class="main">
//!        <p>Content here</p>
//!      </div>
//!    </body>
//!  </html>
//!
//!  6. Custom Drop
//!
//!  Print Dropping node: <tag> to stderr. Demonstrate no reference cycles — all nodes get dropped when root goes out of scope.
//!
//!  7. Tests
//!
//!  - Build 3+ level tree, assert render output
//!
//!  - find_by_tag across levels
//!  - depth for root, middle, leaf
//!  - Remove child → verify it's gone from render, parent ref cleared
//!  - Weak returns None after node dropped
//!  - Deref coercion for TextContent
//!
//!  Acceptance criteria
//!
//!  - cargo build — no warnings
//!  - cargo test — all pass
//!  - All smart pointers used: Box, Rc, RefCell, Weak
//!  - Deref and Drop implemented
//!  - No reference cycles

use std::collections::HashMap;
use std::ops::Deref;
use std::{cell::RefCell, rc::Rc, rc::Weak};

pub type NodeRef = Rc<RefCell<Node>>;

#[derive(Debug)]
pub struct Node {
    pub tag: String,
    attributes: HashMap<String, String>,
    text: Option<TextContent>,
    pub parent: Weak<RefCell<Node>>,
    children: RefCell<Vec<Rc<RefCell<Node>>>>,
}

impl Node {
    /// Creating new Node, DOM element with tag name with Rc and RefCell wrapped
    ///
    /// # Arguments
    ///
    /// * `tag` - tag name that is immutable and used while generation.
    ///
    /// # Returns
    /// Rc<RefCell<Node>>  
    ///
    /// # Examples
    /// ```rust,ignore
    /// use smart_pointers::Node;
    ///
    /// Node::new("html")  
    /// ```
    pub fn new(tag: &str) -> NodeRef {
        Rc::new(RefCell::new(Node {
            tag: tag.to_string(),
            attributes: HashMap::new(),
            text: None,
            parent: Weak::new(),
            children: RefCell::new(vec![]),
        }))
    }

    pub fn add_child(parent: &NodeRef, child: &NodeRef) {
        parent.borrow().children.borrow_mut().push(child.clone());
        child.borrow_mut().parent = Rc::downgrade(parent);
    }

    pub fn set_attribute(&mut self, key: &str, value: &str) {
        self.attributes.insert(key.to_string(), value.to_string());
    }

    pub fn set_text(&mut self, text: &str) {
        self.text = Some(TextContent(Box::from(text)));
    }

    pub fn remove_child(parent: &NodeRef, child: &NodeRef) {
        let index = parent
            .borrow()
            .children
            .borrow()
            .iter()
            .position(|e| Rc::ptr_eq(e, child));

        if let Some(i) = index {
            parent.borrow().children.borrow_mut().remove(i);
            child.borrow_mut().parent = Weak::new();
        }
    }

    pub fn parent(&self) -> Option<NodeRef> {
        self.parent.upgrade()
    }

    pub fn depth(node: &NodeRef) -> usize {
        // loop variant
        // let mut depth = 0;
        // let mut parent = node.parent.clone();
        // while let Some(n) = parent.upgrade() {
        //     parent = n.borrow().parent.clone();
        //     depth += 1;
        // }
        // depth

        std::iter::successors(node.borrow().parent.upgrade(), |n| {
            n.borrow().parent.upgrade()
        })
        .count()
    }

    pub fn find_by_tag(node: &NodeRef, tag: &str) -> Vec<NodeRef> {
        let mut vec = vec![];
        if node.borrow().tag == tag {
            vec.push(node.clone());
        }
        node.borrow()
            .children
            .borrow()
            .iter()
            .for_each(|f| vec.extend(Node::find_by_tag(f, tag)));
        vec
    }

    pub fn render(node: &NodeRef) -> String {
        let mut content = String::new();
        let node_borrow = node.borrow();
        let tag = &node_borrow.tag;

        let indent = String::from("  ");
        let depth = Node::depth(node);

        // put tag and attributes
        let attribute_str = node_borrow
            .attributes
            .iter()
            .fold(String::new(), |attr, (k, v)| format!("{attr} {k}=\"{v}\""));
        content.push_str(format!("{}<{tag}{attribute_str}>", indent.repeat(depth)).as_str());

        // put text of tag
        let text = node_borrow
            .text
            .clone()
            .unwrap_or(TextContent(Box::from("")))
            .0;
        if !node_borrow.children.borrow().is_empty() {
            content.push('\n');
            if !text.is_empty() {
                content.push_str(format!("{}{text}", indent.repeat(depth + 1)).as_str());
                content.push('\n');
            }

            // get tags of all childs
            node_borrow
                .children
                .borrow()
                .iter()
                .for_each(|c| content.push_str(Node::render(c).as_str()));
            content.push_str(format!("{}</{tag}>", indent.repeat(depth)).as_str());
        } else {
            content.push_str(&text);
            content.push_str(format!("</{tag}>").as_str());
        }
        content.push('\n');

        content
    }
}

impl Drop for Node {
    fn drop(&mut self) {
        eprintln!("{} dropped.", self.tag);
        self.parent = Weak::new();
        self.children.borrow_mut().clear();
    }
}

// impl Deref for NodeRef {
//     type Target = Node;
//
//     fn deref(&self) -> &Self::Target {
//         self
//     }
// }

#[derive(PartialEq, Debug, Clone)]
pub struct TextContent(Box<str>);

impl Deref for TextContent {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write;
    use std::result::Result;

    fn setup() -> NodeRef {
        let html = Node::new("html");
        let body = Node::new("body");
        let div = Node::new("div");
        let div1 = Node::new("div");
        let p = Node::new("p");
        body.borrow_mut().set_text("Hello world!");
        div.borrow_mut().set_attribute("id", "description");
        p.borrow_mut()
            .set_text("This page generated for test. Cool! Right?");
        Node::add_child(&html, &body);
        Node::add_child(&body, &div);
        Node::add_child(&div, &p);
        Node::add_child(&div, &div1);
        html
    }

    #[test]
    fn test_new() {
        let html = Node::new("html");
        assert_eq!(html.borrow().tag, "html".to_string());
        assert!(html.borrow().text.is_none());
        assert_eq!(html.borrow().attributes, HashMap::new());
        assert!(html.borrow().children.borrow().is_empty());
        assert!(html.borrow().parent.upgrade().is_none());
    }

    #[test]
    fn test_add_child() {
        let html = Node::new("html");
        let head = Node::new("head");
        Node::add_child(&html, &head);
        assert!(Rc::ptr_eq(&html.borrow().children.borrow()[0], &head));
        assert!(Rc::ptr_eq(&html, &head.borrow().parent.upgrade().unwrap()));
        assert_eq!(
            html.borrow().children.borrow()[0].borrow().tag,
            "head".to_string()
        );
    }

    #[test]
    fn test_set_attribute() {
        let html = setup();
        html.borrow_mut().set_attribute("lang", "en");
        assert_eq!(
            html.borrow().attributes.get("lang"),
            Some(&"en".to_string())
        );
    }

    #[test]
    fn test_set_text() {
        let html = Node::new("html");
        html.borrow_mut().set_text("Some text to test");
        let text = html.borrow();
        assert_eq!(
            *text.text.as_ref().unwrap(),
            TextContent(Box::from("Some text to test"))
        );
    }

    #[test]
    fn test_find_by_tag() {
        let html = setup();
        assert_eq!(Node::find_by_tag(&html, "div")[0].borrow().tag, "div");
        assert_eq!(Node::find_by_tag(&html, "div").len(), 2);
    }

    #[test]
    fn test_depth() {
        let html = setup();
        assert_eq!(Node::depth(&html), 0);
        let children = &html.borrow().children;
        assert_eq!(Node::depth(&children.borrow()[0]), 1);
    }

    #[test]
    fn test_remove_child() {
        let html = setup();
        let body = Node::find_by_tag(&html, "body").first().unwrap().clone();
        Node::remove_child(&html, &body);
        assert_eq!(html.borrow().children.borrow().len(), 0);
        assert!(body.borrow().parent.upgrade().is_none());
    }

    #[test]
    fn test_render() -> Result<(), std::fmt::Error> {
        let html = setup();

        let indent = "  ".to_string();
        let mut output = String::new();
        writeln!(output, "<html>")?;
        writeln!(output, "{indent}<body>")?;
        writeln!(output, "{indent}{indent}Hello world!")?;
        writeln!(output, "{indent}{indent}<div id=\"description\">")?;
        writeln!(
            output,
            "{indent}{indent}{indent}<p>This page generated for test. Cool! Right?</p>"
        )?;
        writeln!(output, "{indent}{indent}{indent}<div></div>")?;
        writeln!(output, "{indent}{indent}</div>")?;
        writeln!(output, "{indent}</body>")?;
        writeln!(output, "</html>")?;

        assert_eq!(Node::render(&html), output);
        Ok(())
    }

    #[test]
    fn test_textcontent_deref() {
        let text = TextContent(Box::from("some bla text"));
        assert!(&text.cmp("some bla text").is_eq());
    }

    #[test]
    fn test_weak_drops() {
        let parent = Node::new("div");
        let child = Node::new("p");
        Node::add_child(&parent, &child);
        let parent_weak = Rc::downgrade(&parent);
        assert!(parent_weak.upgrade().is_some()); // alive
        drop(parent); // drop the only strong reference
        assert!(parent_weak.upgrade().is_none()); // gone
    }
}
