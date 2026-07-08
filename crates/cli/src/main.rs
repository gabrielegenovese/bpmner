use core::bpmn::chor::{Chor, ChorEl};
use core::bpmn::edge::ControlFlow;
use core::encoder::enc::encode_with_init;

fn main() {
    let mut task = Vec::new();
    task.push(ChorEl::Task {
        input: ControlFlow::new("e1"),
        output: ControlFlow::new("e2"),
    });
    let chor = Chor { elements: task };
    println!("{:?}", chor);
    let net = encode_with_init(&chor);
    println!("{:?}", net);
}
