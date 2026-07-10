pub mod bpmn {
    pub mod chor {
        pub mod parser;
        pub mod syntax;
    }
    pub mod collab {
        pub mod parser;
        pub mod syntax;
    }
    pub mod edge;
}

pub mod petri_net {
    pub mod exporter;
    pub mod pn;
}

pub mod encoder {
    pub mod enc_chor;
    pub mod enc_collab;
    pub mod preproc;
    pub mod util;
}
