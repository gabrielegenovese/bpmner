use crate::bpmn::chor::syntax::{Chor, ChorEl};
use crate::bpmn::edge::ControlFlow;
use quick_xml::events::{BytesStart, Event};
use quick_xml::reader::Reader;
use std::collections::{HashMap, HashSet};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("XML parsing error: {0}")]
    Xml(String),

    #[error("missing attribute '{attribute}' in element '{element}'")]
    MissingAttribute { element: String, attribute: String },

    #[error("invalid node shape for '{id}': {reason}")]
    InvalidNodeShape { id: String, reason: String },
}

impl From<quick_xml::Error> for ParseError {
    fn from(e: quick_xml::Error) -> Self {
        ParseError::Xml(e.to_string())
    }
}

#[derive(Debug, Clone)]
struct RawFlow {
    id: String,
    source_ref: String,
    target_ref: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NodeKind {
    Start,
    End,
    Task,
    Exclusive,
    Parallel,
    Inclusive,
}

#[derive(Debug, Clone)]
struct RawNode {
    id: String,
    kind: NodeKind,
}

fn local_name(e: &BytesStart) -> String {
    String::from_utf8_lossy(e.name().local_name().as_ref()).into_owned()
}

fn get_attr(e: &BytesStart, name: &str) -> Option<String> {
    e.attributes()
        .flatten()
        .find(|a| a.key.as_ref() == name.as_bytes())
        .map(|a| String::from_utf8_lossy(&a.value).into_owned())
}

fn require_attr(e: &BytesStart, element: &str, attr: &str) -> Result<String, ParseError> {
    get_attr(e, attr).ok_or_else(|| ParseError::MissingAttribute {
        element: element.to_string(),
        attribute: attr.to_string(),
    })
}

fn node_kind_of(name: &str) -> Option<NodeKind> {
    match name {
        "startEvent" => Some(NodeKind::Start),
        "endEvent" => Some(NodeKind::End),
        "choreographyTask" => Some(NodeKind::Task),
        "exclusiveGateway" => Some(NodeKind::Exclusive),
        "parallelGateway" => Some(NodeKind::Parallel),
        "inclusiveGateway" => Some(NodeKind::Inclusive),
        _ => None,
    }
}

fn collect_raw(xml: &str) -> Result<(Vec<RawNode>, Vec<RawFlow>), ParseError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut nodes = Vec::new();
    let mut flows = Vec::new();

    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(e) | Event::Empty(e) => {
                let name = local_name(&e);

                if name == "sequenceFlow" {
                    flows.push(RawFlow {
                        id: require_attr(&e, "sequenceFlow", "id")?,
                        source_ref: require_attr(&e, "sequenceFlow", "sourceRef")?,
                        target_ref: require_attr(&e, "sequenceFlow", "targetRef")?,
                    });
                } else if let Some(kind) = node_kind_of(&name) {
                    nodes.push(RawNode {
                        id: require_attr(&e, &name, "id")?,
                        kind,
                    });
                }
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok((nodes, flows))
}

fn index_flows(
    flows: &[RawFlow],
) -> (
    HashMap<String, HashSet<ControlFlow>>,
    HashMap<String, HashSet<ControlFlow>>,
) {
    let outgoing = flows.iter().fold(HashMap::new(), |mut acc, f| {
        acc.entry(f.source_ref.clone())
            .or_insert_with(HashSet::new)
            .insert(ControlFlow::new(f.id.clone()));
        acc
    });

    let incoming = flows.iter().fold(HashMap::new(), |mut acc, f| {
        acc.entry(f.target_ref.clone())
            .or_insert_with(HashSet::new)
            .insert(ControlFlow::new(f.id.clone()));
        acc
    });

    (incoming, outgoing)
}

fn exactly_one(
    set: HashSet<ControlFlow>,
    node_id: &str,
    dir: &str,
) -> Result<ControlFlow, ParseError> {
    let mut it = set.into_iter();
    match (it.next(), it.next()) {
        (Some(cf), None) => Ok(cf),
        _ => Err(ParseError::InvalidNodeShape {
            id: node_id.to_string(),
            reason: format!("expected exactly one {dir} control flow"),
        }),
    }
}

fn classify_gateway(
    node: &RawNode,
    incoming: HashSet<ControlFlow>,
    outgoing: HashSet<ControlFlow>,
) -> Result<ChorEl, ParseError> {
    match (incoming.len(), outgoing.len(), node.kind) {
        (1, n, NodeKind::Exclusive) if n > 1 => Ok(ChorEl::XorSplit {
            input: exactly_one(incoming, &node.id, "incoming")?,
            output: outgoing,
        }),
        (n, 1, NodeKind::Exclusive) if n > 1 => Ok(ChorEl::XorJoin {
            input: incoming,
            output: exactly_one(outgoing, &node.id, "outgoing")?,
        }),
        (1, n, NodeKind::Parallel) if n > 1 => Ok(ChorEl::AndSplit {
            input: exactly_one(incoming, &node.id, "incoming")?,
            output: outgoing,
        }),
        (n, 1, NodeKind::Parallel) if n > 1 => Ok(ChorEl::AndJoin {
            input: incoming,
            output: exactly_one(outgoing, &node.id, "outgoing")?,
        }),
        (1, n, NodeKind::Inclusive) if n > 1 => Ok(ChorEl::OrSplit {
            input: exactly_one(incoming, &node.id, "incoming")?,
            output: outgoing,
        }),
        (n, 1, NodeKind::Inclusive) if n > 1 => Ok(ChorEl::OrJoin {
            input: incoming,
            output: exactly_one(outgoing, &node.id, "outgoing")?,
        }),
        (i, o, _) => Err(ParseError::InvalidNodeShape {
            id: node.id.clone(),
            reason: format!(
                "gateway must be a split (1 in, >1 out) or a join (>1 in, 1 out); got {i} incoming, {o} outgoing"
            ),
        }),
    }
}

fn node_to_chor_el(
    node: &RawNode,
    incoming: HashSet<ControlFlow>,
    outgoing: HashSet<ControlFlow>,
) -> Result<ChorEl, ParseError> {
    match node.kind {
        NodeKind::Start => Ok(ChorEl::Start {
            output: exactly_one(outgoing, &node.id, "outgoing")?,
        }),
        NodeKind::End => Ok(ChorEl::End {
            input: exactly_one(incoming, &node.id, "incoming")?,
        }),
        NodeKind::Task => Ok(ChorEl::Task {
            input: exactly_one(incoming, &node.id, "incoming")?,
            output: exactly_one(outgoing, &node.id, "outgoing")?,
        }),
        NodeKind::Exclusive | NodeKind::Parallel | NodeKind::Inclusive => {
            classify_gateway(node, incoming, outgoing)
        }
    }
}

pub fn parse(xml: &str) -> Result<Chor, ParseError> {
    let (nodes, flows) = collect_raw(xml)?;
    let (incoming, outgoing) = index_flows(&flows);

    let elements: Result<Vec<ChorEl>, ParseError> = nodes
        .iter()
        .map(|node| {
            let ins = incoming.get(&node.id).cloned().unwrap_or_default();
            let outs = outgoing.get(&node.id).cloned().unwrap_or_default();
            node_to_chor_el(node, ins, outs)
        })
        .collect();

    Ok(Chor {
        elements: elements?,
    })
}
