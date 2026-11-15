use std::collections::{HashMap, HashSet};

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
    /// Initial marking: number of tokens per place.
    pub marking: HashMap<Place, usize>,
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

    /// Adds `tokens` tokens to place `p` in the initial marking.
    pub fn mark(mut self, p: impl Into<Place>, tokens: usize) -> Self {
        let p = p.into();
        self.places.insert(p.clone());
        *self.marking.entry(p).or_insert(0) += tokens;
        self
    }

    pub fn token_count(&self, p: &str) -> usize {
        self.marking.get(p).copied().unwrap_or(0)
    }

    pub fn union(mut self, other: PetriNet) -> PetriNet {
        self.places.extend(other.places);
        self.transitions.extend(other.transitions);
        self.flow.extend(other.flow);

        for (place, tokens) in other.marking {
            *self.marking.entry(place).or_insert(0) += tokens;
        }

        self
    }
}

impl std::ops::Add for PetriNet {
    type Output = PetriNet;
    fn add(self, rhs: PetriNet) -> PetriNet {
        self.union(rhs)
    }
}
