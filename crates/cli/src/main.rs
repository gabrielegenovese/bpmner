use core::bpmn::chor::{Choreography, ChoreographyEl};
use core::bpmn::edge::ControlFlow;
use core::encoder::enc::encode_with_init;

fn main() {
    let mut task = Vec::new();
    task.push(ChoreographyEl::Task {
        input: ControlFlow::new("e1"),
        output: ControlFlow::new("e2"),
    });
    let chor = Choreography { elements: task };
    println!("{:?}", chor);
    let net = encode_with_init(&chor);
    println!("{:?}", net);
}
