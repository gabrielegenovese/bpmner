use core::bpmn::chor::parser::parse;
use core::encoder::enc_chor::encode;
use core::petri_net::exporter::{export_to_dot, export_to_pnml};

fn main() {
    let chor = parse(include_str!("../../../examples/chor/base.bpmn")).unwrap();
    println!("{:?}", chor);
    let net = encode(&chor).unwrap();
    println!("{:?}", net);
    export_to_pnml("output/test.pnml", &net).unwrap(); // todo: test
    export_to_dot("output/test.dot", &net).unwrap();
}
