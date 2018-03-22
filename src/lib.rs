#[macro_use]
extern crate error_chain;
extern crate html5ever;
extern crate markup5ever;
extern crate reqwest;

pub mod errors;

use std::default::Default;
use std::rc::Rc;
use std::iter::Iterator;
use html5ever::parse_document;
use html5ever::driver::ParseOpts;
use html5ever::rcdom::RcDom;
use html5ever::tendril::TendrilSink;
use html5ever::tree_builder::TreeBuilderOpts;
use markup5ever::rcdom::{Node, NodeData};

const IQDB_URL: &str = "https://iqdb.org";

#[derive(Debug)]
pub struct Service {
    value: i32,
    name: String,
    url: String,
}

#[derive(Debug)]
pub enum MatchType {
    Best,
    Possible,
}

#[derive(Debug)]
pub struct Match {
    typ: MatchType,
    url: String,
    similarity: i32,
}

pub fn available_services() -> errors::Result<Vec<Service>> {
    let text = reqwest::get(IQDB_URL)?.text()?;
    let opts = ParseOpts {
        tree_builder: TreeBuilderOpts {
            drop_doctype: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let dom = parse_document(RcDom::default(), opts)
        .from_utf8()
        .read_from(&mut text.as_bytes())?;
    let html = node_children(&dom.document, "html", &[]);
    let body = node_children(&html[0], "body", &[]);
    let form = node_children(&body[0], "form", &[]);
    let table = node_children(&form[0], "table", &[]);
    let tbody = node_children(&table[0], "tbody", &[]);
    let tr = node_children(&tbody[0], "tr", &[]);
    let mut services = vec![];
    for t in tr {
        let th = node_children(&t, "th", &[]);
        let label = node_children(&th[0], "label", &[]);
        let input = node_children(&label[0], "input", &[]);
        let a = node_children(&label[0], "a", &[]);
        let value: i32 = attribute_value(&input[0], "value").parse().unwrap();
        let url = prepend_https(&attribute_value(&a[0], "href"));
        let name = node_text(&a[0]);
        services.push(Service {
            value: value,
            name: name,
            url: url,
        });
    }

    Ok(services)
}

pub fn search_by_url(image_url: &str, services: &[Service]) -> errors::Result<Vec<Match>> {
    let mut params = vec![("url", image_url.to_string())];
    services
        .iter()
        .for_each(|service| params.push(("service[]", service.value.to_string())));
    let client = reqwest::Client::new();
    let text = client.post(IQDB_URL).form(&params).send()?.text()?;
    let opts = ParseOpts {
        tree_builder: TreeBuilderOpts {
            drop_doctype: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let dom = parse_document(RcDom::default(), opts)
        .from_utf8()
        .read_from(&mut text.as_bytes())?;
    let html = node_children(&dom.document, "html", &[]);
    let body = node_children(&html[0], "body", &[]);
    let div0 = node_children(&body[0], "div", &[("id", "pages")]);
    let div1 = node_children(&div0[0], "div", &[]);
    let mut matches = vec![];
    for div in div1 {
        let table = node_children(&div, "table", &[]);
        let tbody = node_children(&table[0], "tbody", &[]);
        let tr = node_children(&tbody[0], "tr", &[]);
        let th = node_children(&tr[0], "th", &[]);
        let th_text = node_text(&th[0]).to_lowercase();
        if th_text.contains("best") || th_text.contains("possible") {
            let td = node_children(&tr[1], "td", &[]);
            let a = node_children(&td[0], "a", &[]);
            let url = prepend_https(&attribute_value(&a[0], "href"));
            let td = node_children(&tr[tr.len() - 1], "td", &[]);
            let td_text = node_text(&td[0]);
            let similarity: i32 = td_text[..td_text.find('%').unwrap()].parse().unwrap();
            matches.push(Match {
                typ: if th_text.contains("best") {
                    MatchType::Best
                } else {
                    MatchType::Possible
                },
                url: url,
                similarity: similarity,
            });
        }
    }

    Ok(matches)
}

fn node_children(node: &Node, tag: &str, attribute_values: &[(&str, &str)]) -> Vec<Rc<Node>> {
    node.children
        .borrow()
        .iter()
        .filter(|child| match child.data {
            NodeData::Element {
                ref name,
                attrs: _,
                template_contents: _,
                mathml_annotation_xml_integration_point: _,
            } => {
                if name.local.as_ref() != tag {
                    return false;
                }
                for &(attribute, value) in attribute_values {
                    if attribute_value(child, attribute).as_str() != value {
                        return false;
                    }
                }
                true
            }
            _ => false,
        })
        .cloned()
        .collect()
}

fn node_text(node: &Node) -> String {
    node.children
        .borrow()
        .iter()
        .filter(|child| match child.data {
            NodeData::Text { contents: _ } => true,
            _ => false,
        })
        .map(|child| match child.data {
            NodeData::Text { ref contents } => {
                String::from_utf8_lossy(contents.borrow().as_bytes()).to_string()
            }
            _ => unreachable!(),
        })
        .next()
        .unwrap()
}

fn attribute_value(node: &Node, attribute: &str) -> String {
    match node.data {
        NodeData::Element {
            name: _,
            ref attrs,
            template_contents: _,
            mathml_annotation_xml_integration_point: _,
        } => attrs
            .borrow()
            .iter()
            .filter(|attr| attr.name.local.as_ref() == attribute)
            .map(|attr| {
                String::from_utf8_lossy(attr.value.as_bytes())
                    .to_string()
                    .parse()
                    .unwrap()
            })
            .next()
            .unwrap_or_else(|| String::new()),
        _ => unreachable!(),
    }
}

fn prepend_https(url: &str) -> String {
    if &url[..2] == "//" {
        "http:".to_string() + url
    } else {
        url.to_string()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
