use nom::AsBytes;
use xml_builder::{XMLBuilder, XMLElement, XMLVersion};

use crate::decode::Candidate;
use crate::tree::{Information, Leaf, Node, Tree};

pub fn tree_to_xml(candidate: &Candidate) {
    let mut xml = XMLBuilder::new()
        .version(XMLVersion::XML1_1)
        .encoding("UTF-8".into())
        .build();

    let mut nodes = XMLElement::new("tree");

    nodes_to_xml(
        &candidate.annotations,
        candidate.data.as_bytes(),
        &mut nodes,
    );

    xml.set_root_element(nodes);

    xml.generate(std::io::stdout()).unwrap();
}

fn nodes_to_xml(nodes: &[Node], data: &[u8], element: &mut XMLElement) {
    for node in nodes {
        element.add_child(node_to_xml(node, data)).unwrap();
    }
}

fn node_to_xml(node: &Node, data: &[u8]) -> XMLElement {
    match node {
        Node::Group {
            path,
            location: _,
            information,
            children,
        } => {
            let mut element = XMLElement::new("group");
            let mut path_el = XMLElement::new("path");
            path_el.add_text(path.join("/")).unwrap();
            element.add_child(path_el).unwrap();
            attach_information(information, &mut element);
            let mut children_element = XMLElement::new("children");
            nodes_to_xml(children, data, &mut children_element);
            element.add_child(children_element).unwrap();
            element
        }
        Node::Leaf(Leaf::Real(leaf)) => {
            let mut element = XMLElement::new("leaf");
            attach_data(&data[leaf.location.range()], &mut element);
            attach_information(&leaf.information, &mut element);
            element
        }
        Node::Leaf(Leaf::Virtual(leaf)) => {
            let mut element = XMLElement::new("leaf");
            element.add_attribute("virtual", "true");
            attach_information(&leaf.information, &mut element);
            element
        }
    }
}

fn attach_data(data: &[u8], element: &mut XMLElement) {
    let mut data_el = XMLElement::new("data");
    data_el.add_text(hex::encode(data)).unwrap();
    element.add_child(data_el).unwrap();
}

fn attach_information(information: &Information, element: &mut XMLElement) {
    let mut value = XMLElement::new("value");
    value.add_text(information.value.preview()).unwrap();
    element.add_child(value).unwrap();

    let mut label = XMLElement::new("label");
    label.add_text(information.label.clone()).unwrap();
    element.add_child(label).unwrap();

    if let Some(doc) = information.doc.clone() {
        let mut el = XMLElement::new("doc");
        el.add_text(doc).unwrap();
        element.add_child(el).unwrap();
    }

    if let Some(splain) = information.splain.clone() {
        let mut el = XMLElement::new("splain");
        el.add_text(splain).unwrap();
        element.add_child(el).unwrap();
    }

    if !information.tags.is_empty() {
        let mut tags = XMLElement::new("tags");
        for t in &information.tags {
            let mut tag = XMLElement::new("tag");

            let mut label = XMLElement::new("label");
            label.add_text(t.label.clone()).unwrap();
            tag.add_child(label).unwrap();

            if let Some(color) = &t.color {
                let mut el = XMLElement::new("color");
                el.add_text(color.clone()).unwrap();
                tag.add_child(el).unwrap();
            }

            if let Some(doc) = &t.doc {
                let mut el = XMLElement::new("doc");
                el.add_text(doc.clone()).unwrap();
                tag.add_child(el).unwrap();
            }

            tags.add_child(tag).unwrap();
        }
        element.add_child(tags).unwrap();
    }
}
