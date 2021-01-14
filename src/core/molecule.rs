use crate::core::Element;
use petgraph::prelude::*;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Atom {
    pub element: Element,
    //pub formal_charge: i8,
    //pub explicit_hydrogens: u8,
    //pub map: Option<i8>,
    //pub electrons: u8
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Bond {
    Single,
    Double,
    Triple,
    Aromatic,
}

#[derive(Clone, Debug)]
pub struct Molecule {
    graph: petgraph::graph::UnGraph<Atom, Bond>,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct AtomIndex(petgraph::graph::NodeIndex);

// TODO: Proper error handling
impl Molecule {
    pub fn new() -> Self {
        Molecule {
            graph: petgraph::graph::Graph::new_undirected(),
        }
    }

    pub fn add_atom(&mut self, atom: Atom) -> AtomIndex {
        AtomIndex(self.graph.add_node(atom))
    }

    pub fn add_bond(&mut self, a: AtomIndex, b: AtomIndex, bond: Bond) -> Result<(), ()> {
        if self.graph.contains_edge(a.0, b.0) {
            Err(())
        } else {
            self.graph.add_edge(a.0, b.0, bond);
            Ok(())
        }
    }

    pub fn atom(&self, atom: AtomIndex) -> &Atom {
        &self.graph[atom.0]
    }

    //pub fn atom_mut(&mut self, atom: AtomIndex) -> &mut Atom {
    //&mut self.graph[atom.0]
    //}
}

impl PartialEq<Molecule> for Molecule {
    fn eq(&self, other: &Molecule) -> bool {
        petgraph::algo::is_isomorphic_matching(
            &self.graph,
            &other.graph,
            |na, nb| na == nb,
            |ea, eb| ea == eb,
        )
    }
}
