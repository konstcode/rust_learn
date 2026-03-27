use smart_pointers::{Node, NodeRef};

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

fn main() {
    let html = setup();

    let p = Node::find_by_tag(&html, "p")[0].clone();
    if p.borrow().tag == "p" {
        println!("Found tags p.");
    }

    let div = Node::find_by_tag(&html, "div")[0].clone();

    println!(
        "Depth for tags html: {}; div: {}; p: {}.",
        Node::depth(&html),
        Node::depth(&div),
        Node::depth(&p)
    );

    println!("{}", Node::render(&html));

    Node::remove_child(&div, &p);

    if p.borrow().parent.upgrade().is_none() {
        println!("p was removed from div");
    };

    println!("{}", Node::render(&html));
}
