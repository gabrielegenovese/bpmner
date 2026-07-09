use core::bpmn::chor::{Chor, ChorEl};
use core::bpmn::edge::ControlFlow;
use core::encoder::enc::encode_with_init;
use core::petri_net::exporter::{export_to_dot, export_to_pnml};
use std::collections::HashSet;

fn main() {
    let mut el = Vec::new();
    el.push(ChorEl::XorSplit {
        input: ControlFlow::new("e1"),
        output: HashSet::from([ControlFlow::new("e2"), ControlFlow::new("e3")]),
    });
    el.push(ChorEl::End {
        input: ControlFlow::new("e2"),
    });
    // el.push(ChorEl::End {
    //     input: ControlFlow::new("e3"),
    // });
    let chor = Chor { elements: el };
    println!("{:?}", chor);
    let net = encode_with_init(&chor);
    println!("{:?}", net);
    export_to_pnml("test.pnml", &net);
    export_to_dot("test.dot", &net);
}
