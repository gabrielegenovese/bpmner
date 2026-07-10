use core::bpmn::chor::parser::parse;
use core::encoder::enc_chor::encode;
use core::petri_net::exporter::{export_to_dot, export_to_pnml};

fn main() {
    // let mut el = Vec::new();
    // el.push(ChorEl::OrSplit {
    //     input: ControlFlow::new("e1"),
    //     output: HashSet::from([ControlFlow::new("e2"), ControlFlow::new("e3")]),
    // });
    // el.push(ChorEl::End {
    //     input: ControlFlow::new("e2"),
    // });
    // el.push(ChorEl::End {
    //     input: ControlFlow::new("e3"),
    // });
    // let chor = Chor { elements: el };
    let chor = parse(include_str!("../../../examples/chor/base.bpmn")).unwrap();
    println!("{:?}", chor);
    let net = encode(&chor);
    println!("{:?}", net);
    export_to_pnml("output/test.pnml", &net).unwrap(); // todo: test
    export_to_dot("output/test.dot", &net).unwrap();
}
