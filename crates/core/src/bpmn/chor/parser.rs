use crate::bpmn::chor::syntax::{Chor, ChorEl};
use crate::bpmn::edge::ControlFlow;
use quick_xml::events::{BytesStart, Event};
use quick_xml::reader::Reader;
use std::collections::{HashMap, HashSet};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("XML parsing error at byte {position}: {message}")]
    Xml { position: u64, message: String },

    #[error("missing attribute '{attribute}' in element '{element}'")]
    MissingAttribute { element: String, attribute: String },

    #[error("invalid node shape for '{id}': {reason}")]
    InvalidNodeShape { id: String, reason: String },

    #[error(
        "this document is a BPMN collaboration, not a choreography; \
         collaborations (participants and message flows between pools) are not \
         supported yet"
    )]
    UnsupportedCollaboration,

    #[error(
        "this document contains a BPMN process, not a choreography; \
         process models are not supported yet"
    )]
    UnsupportedProcess,

    #[error("no BPMN choreography found: the document must contain a <choreography> element")]
    MissingChoreography,

    #[error("the choreography is empty: no supported flow nodes were found")]
    EmptyChoreography,

    #[error("unsupported BPMN element '{element}' (id '{id}'): {reason}")]
    UnsupportedElement {
        element: String,
        id: String,
        reason: &'static str,
    },

    #[error("duplicate id '{id}' is used by more than one element")]
    DuplicateId { id: String },

    #[error("sequence flow '{flow}' references unknown {side} node '{node}'")]
    UnknownReference {
        flow: String,
        side: &'static str,
        node: String,
    },
}

impl From<quick_xml::Error> for ParseError {
    fn from(e: quick_xml::Error) -> Self {
        ParseError::Xml {
            position: 0,
            message: e.to_string(),
        }
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

/// BPMN elements that cannot be encoded by this tool. Detecting them early
/// produces a much clearer error than silently ignoring them and failing later
/// with a mysterious malformed choreography.
fn unsupported_element_reason(name: &str) -> Option<&'static str> {
    match name {
        "subChoreography" => Some("sub-choreographies are not supported yet"),
        "callChoreography" => Some("called choreographies are not supported yet"),
        "eventBasedGateway" => Some(
            "event-based gateways are not supported yet; only exclusive, parallel and inclusive gateways are encoded",
        ),
        "complexGateway" => Some("complex gateways are not supported"),
        "boundaryEvent" => Some("boundary events are not supported"),
        "intermediateCatchEvent" | "intermediateThrowEvent" => {
            Some("intermediate events are not supported yet")
        }
        "subProcess" | "transaction" | "callActivity" => {
            Some("embedded and called sub-processes are not supported")
        }
        "task" | "businessRuleTask" | "manualTask" | "userTask" | "serviceTask" | "scriptTask"
        | "sendTask" | "receiveTask" => Some(
            "collaboration tasks are not supported; use choreographyTask inside a choreography",
        ),
        _ => None,
    }
}

struct Collected {
    nodes: Vec<RawNode>,
    flows: Vec<RawFlow>,
    has_choreography: bool,
    has_collaboration: bool,
    has_process: bool,
    unsupported: Option<(String, String, &'static str)>,
}

fn collect_raw(xml: &str) -> Result<Collected, ParseError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut nodes = Vec::new();
    let mut flows = Vec::new();
    let mut seen_ids: HashMap<String, ()> = HashMap::new();

    let mut has_choreography = false;
    let mut has_collaboration = false;
    let mut has_process = false;
    let mut unsupported = None;

    loop {
        let event = reader
            .read_event_into(&mut buf)
            .map_err(|e| ParseError::Xml {
                position: reader.buffer_position(),
                message: e.to_string(),
            })?;

        match event {
            Event::Start(e) | Event::Empty(e) => {
                let name = local_name(&e);

                match name.as_str() {
                    "choreography" => has_choreography = true,
                    "collaboration" => has_collaboration = true,
                    "process" => has_process = true,
                    _ => {}
                }

                if let Some(reason) = unsupported_element_reason(&name)
                    && unsupported.is_none()
                {
                    let id = get_attr(&e, "id").unwrap_or_else(|| "<unknown>".to_string());
                    // Do not fail immediately: a collaboration should still be
                    // reported as such, even if it also contains unsupported
                    // process elements.
                    unsupported = Some((name.clone(), id, reason));
                }

                if name == "sequenceFlow" {
                    let id = require_attr(&e, "sequenceFlow", "id")?;
                    let flow = RawFlow {
                        id: id.clone(),
                        source_ref: require_attr(&e, "sequenceFlow", "sourceRef")?,
                        target_ref: require_attr(&e, "sequenceFlow", "targetRef")?,
                    };

                    if seen_ids.insert(id.clone(), ()).is_some() {
                        return Err(ParseError::DuplicateId { id });
                    }

                    flows.push(flow);
                } else if let Some(kind) = node_kind_of(&name) {
                    let id = require_attr(&e, &name, "id")?;

                    if seen_ids.insert(id.clone(), ()).is_some() {
                        return Err(ParseError::DuplicateId { id });
                    }

                    nodes.push(RawNode { id, kind });
                }
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(Collected {
        nodes,
        flows,
        has_choreography,
        has_collaboration,
        has_process,
        unsupported,
    })
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
        (None, _) => Err(ParseError::InvalidNodeShape {
            id: node_id.to_string(),
            reason: format!("missing {dir} control flow"),
        }),
        (Some(_), Some(_)) => Err(ParseError::InvalidNodeShape {
            id: node_id.to_string(),
            reason: format!("expected exactly one {dir} control flow, found more than one"),
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
                "a gateway must be either a split (1 incoming, >1 outgoing) or a join (>1 incoming, 1 outgoing); got {i} incoming and {o} outgoing control flows"
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

fn check_flow_references(nodes: &[RawNode], flows: &[RawFlow]) -> Result<(), ParseError> {
    let node_ids: HashSet<&str> = nodes.iter().map(|node| node.id.as_str()).collect();

    if let Some(flow) = flows
        .iter()
        .find(|flow| !node_ids.contains(flow.source_ref.as_str()))
    {
        return Err(ParseError::UnknownReference {
            flow: flow.id.clone(),
            side: "source",
            node: flow.source_ref.clone(),
        });
    }

    if let Some(flow) = flows
        .iter()
        .find(|flow| !node_ids.contains(flow.target_ref.as_str()))
    {
        return Err(ParseError::UnknownReference {
            flow: flow.id.clone(),
            side: "target",
            node: flow.target_ref.clone(),
        });
    }

    Ok(())
}

pub fn parse(xml: &str) -> Result<Chor, ParseError> {
    let collected = collect_raw(xml)?;

    if collected.has_collaboration {
        return Err(ParseError::UnsupportedCollaboration);
    }

    if !collected.has_choreography {
        return Err(if collected.has_process {
            ParseError::UnsupportedProcess
        } else {
            ParseError::MissingChoreography
        });
    }

    if let Some((element, id, reason)) = collected.unsupported {
        return Err(ParseError::UnsupportedElement {
            element,
            id,
            reason,
        });
    }

    if collected.nodes.is_empty() {
        return Err(ParseError::EmptyChoreography);
    }

    check_flow_references(&collected.nodes, &collected.flows)?;

    let (incoming, outgoing) = index_flows(&collected.flows);

    let elements: Result<Vec<ChorEl>, ParseError> = collected
        .nodes
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

#[cfg(test)]
mod tests {
    use super::*;

    fn wrap(inner: &str) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL">
  <bpmn:choreography id="Chor_1">
    {inner}
  </bpmn:choreography>
</bpmn:definitions>"#
        )
    }

    #[test]
    fn parses_a_minimal_choreography() {
        let xml = wrap(
            r#"
    <bpmn:startEvent id="start" />
    <bpmn:choreographyTask id="task" />
    <bpmn:endEvent id="end" />
    <bpmn:sequenceFlow id="f1" sourceRef="start" targetRef="task" />
    <bpmn:sequenceFlow id="f2" sourceRef="task" targetRef="end" />
"#,
        );

        let chor = parse(&xml).expect("choreography should parse");
        assert_eq!(chor.elements.len(), 3);
    }

    #[test]
    fn rejects_collaborations() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL">
  <bpmn:collaboration id="Collab_1">
    <bpmn:participant id="p1" />
  </bpmn:collaboration>
</bpmn:definitions>"#;

        assert!(matches!(
            parse(xml),
            Err(ParseError::UnsupportedCollaboration)
        ));
    }

    #[test]
    fn rejects_process_only_models() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL">
  <bpmn:process id="Process_1">
    <bpmn:startEvent id="start" />
  </bpmn:process>
</bpmn:definitions>"#;

        assert!(matches!(parse(xml), Err(ParseError::UnsupportedProcess)));
    }

    #[test]
    fn rejects_unknown_flow_references() {
        let xml = wrap(
            r#"
    <bpmn:startEvent id="start" />
    <bpmn:endEvent id="end" />
    <bpmn:sequenceFlow id="f1" sourceRef="start" targetRef="missing" />
"#,
        );

        assert!(matches!(
            parse(&xml),
            Err(ParseError::UnknownReference { .. })
        ));
    }

    #[test]
    fn rejects_duplicate_ids() {
        let xml = wrap(
            r#"
    <bpmn:startEvent id="dup" />
    <bpmn:endEvent id="dup" />
"#,
        );

        assert!(matches!(parse(&xml), Err(ParseError::DuplicateId { .. })));
    }

    #[test]
    fn rejects_unsupported_elements() {
        let xml = wrap(r#"<bpmn:eventBasedGateway id="gw" />"#);

        assert!(matches!(
            parse(&xml),
            Err(ParseError::UnsupportedElement { .. })
        ));
    }
}
