use std::collections::HashSet;

pub type Place = String;

pub type Transition = String;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Arc {
    PT(Place, Transition), // (p, t) in F
    TP(Transition, Place), // (t, p) in F
}

#[derive(Debug, Clone, Default)]
pub struct PetriNet {
    pub places: HashSet<Place>,
    pub transitions: HashSet<Transition>,
    pub flow: HashSet<Arc>,
}

impl PetriNet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn arc_pt(mut self, p: impl Into<Place>, t: impl Into<Transition>) -> Self {
        let p = p.into();
        let t = t.into();
        self.places.insert(p.clone());
        self.transitions.insert(t.clone());
        self.flow.insert(Arc::PT(p, t));
        self
    }

    pub fn arc_tp(mut self, t: impl Into<Transition>, p: impl Into<Place>) -> Self {
        let t = t.into();
        let p = p.into();
        self.transitions.insert(t.clone());
        self.places.insert(p.clone());
        self.flow.insert(Arc::TP(t, p));
        self
    }

    pub fn union(mut self, other: PetriNet) -> PetriNet {
        self.places.extend(other.places);
        self.transitions.extend(other.transitions);
        self.flow.extend(other.flow);
        self
    }
}

impl std::ops::Add for PetriNet {
    type Output = PetriNet;
    fn add(self, rhs: PetriNet) -> PetriNet {
        self.union(rhs)
    }
}
