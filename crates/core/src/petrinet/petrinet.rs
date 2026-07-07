//! Core Petri net library.
use std::collections::{HashMap, HashSet};

/// A place identifier.
pub type Place = String;

/// A transition identifier.
pub type Transition = String;

/// A Petri net N = (P, T, F).
///
/// The flow relation F is stored split into `pre` (P × T, i.e. arcs into
/// transitions) and `post` (T × P, i.e. arcs out of transitions) for
/// efficient preset/postset queries, mirroring how •t and t• are used
/// throughout the paper's encoding and proofs.
#[derive(Debug, Clone, Default)]
pub struct PetriNet {
    places: HashSet<Place>,
    transitions: HashSet<Transition>,
    /// arcs (p, t) ∈ F, i.e. p ∈ •t
    pre: HashSet<(Place, Transition)>,
    /// arcs (t, p) ∈ F, i.e. p ∈ t•
    post: HashSet<(Transition, Place)>,
}

impl PetriNet {
    pub fn empty() -> Self {
        PetriNet::default()
    }

    /// Add a place to P (idempotent).
    pub fn add_place(&mut self, p: impl Into<Place>) {
        self.places.insert(p.into());
    }

    /// Add a transition to T (idempotent).
    pub fn add_transition(&mut self, t: impl Into<Transition>) {
        self.transitions.insert(t.into());
    }

    /// Add a flow arc (p, t) ∈ F, i.e. place p feeds transition t.
    /// Implicitly registers p and t if not already present.
    pub fn add_arc_place_to_transition(&mut self, p: impl Into<Place>, t: impl Into<Transition>) {
        let p = p.into();
        let t = t.into();
        self.places.insert(p.clone());
        self.transitions.insert(t.clone());
        self.pre.insert((p, t));
    }

    /// Add a flow arc (t, p) ∈ F, i.e. transition t produces on place p.
    /// Implicitly registers p and t if not already present.
    pub fn add_arc_transition_to_place(&mut self, t: impl Into<Transition>, p: impl Into<Place>) {
        let t = t.into();
        let p = p.into();
        self.transitions.insert(t.clone());
        self.places.insert(p.clone());
        self.post.insert((t, p));
    }

    /// Convenience: register a transition together with its full preset
    /// and postset in one call. This is the natural shape for the
    /// table-driven encodings PN(·) in the paper (e.g. PN(task(e,e')),
    /// PN(andSplit(e,E)), PN(orSplit(e,E)), ...).
    pub fn add_transition_with_flow(
        &mut self,
        t: impl Into<Transition>,
        preset: impl IntoIterator<Item = Place>,
        postset: impl IntoIterator<Item = Place>,
    ) {
        let t = t.into();
        self.add_transition(t.clone());
        for p in preset {
            self.add_arc_place_to_transition(p, t.clone());
        }
        for p in postset {
            self.add_arc_transition_to_place(t.clone(), p);
        }
    }

    /// The set of places P.
    pub fn places(&self) -> &HashSet<Place> {
        &self.places
    }

    /// The set of transitions T.
    pub fn transitions(&self) -> &HashSet<Transition> {
        &self.transitions
    }

    /// Preset of a transition: •t = { p ∈ P | (p,t) ∈ F }.
    pub fn preset(&self, t: &str) -> HashSet<Place> {
        self.pre
            .iter()
            .filter(|(_, tt)| tt == t)
            .map(|(p, _)| p.clone())
            .collect()
    }

    /// Postset of a transition: t• = { p ∈ P | (t,p) ∈ F }.
    pub fn postset(&self, t: &str) -> HashSet<Place> {
        self.post
            .iter()
            .filter(|(tt, _)| tt == t)
            .map(|(_, p)| p.clone())
            .collect()
    }

    /// True iff transition `t` is enabled under marking `m`, i.e.
    /// m(p) ≥ 1 for all p ∈ •t.
    pub fn is_enabled(&self, t: &str, m: &Marking) -> bool {
        self.pre
            .iter()
            .filter(|(_, tt)| tt == t)
            .all(|(p, _)| m.get(p) >= 1)
    }

    /// All transitions enabled under marking `m`.
    pub fn enabled_transitions<'a>(&'a self, m: &'a Marking) -> impl Iterator<Item = &'a Transition> + 'a {
        self.transitions
            .iter()
            .filter(move |t| self.is_enabled(t, m))
    }

    /// Union of two Petri nets, N ⊕ N' = (P ∪ P', T ∪ T', F ∪ F'),
    /// per Definition 3. This is how the compositional BPMN encoding
    /// combines the nets of C1 and C2 for a term (C1 | C2).
    pub fn union(&self, other: &PetriNet) -> PetriNet {
        let mut result = self.clone();
        result.places.extend(other.places.iter().cloned());
        result.transitions.extend(other.transitions.iter().cloned());
        result.pre.extend(other.pre.iter().cloned());
        result.post.extend(other.post.iter().cloned());
        result
    }

    /// In-place union: mutate `self` to be self ⊕ other.
    pub fn union_in_place(&mut self, other: &PetriNet) {
        self.places.extend(other.places.iter().cloned());
        self.transitions.extend(other.transitions.iter().cloned());
        self.pre.extend(other.pre.iter().cloned());
        self.post.extend(other.post.iter().cloned());
    }

    /// Prefix every place and transition name with `prefix`. Useful for
    /// building "overlined" dead-propagation copies of a net (Definition
    /// 5): e.g. `overline(net, "bar_")` renames place "e5" to "bar_e5"
    /// and transition "t" to "bar_t", preserving the flow structure.
    pub fn renamed(&self, prefix: &str) -> PetriNet {
        let rename = |s: &str| format!("{prefix}{s}");
        let mut result = PetriNet::empty();
        for p in &self.places {
            result.add_place(rename(p));
        }
        for t in &self.transitions {
            result.add_transition(rename(t));
        }
        for (p, t) in &self.pre {
            result.pre.insert((rename(p), rename(t)));
        }
        for (t, p) in &self.post {
            result.post.insert((rename(t), rename(p)));
        }
        result
    }
}
