// http://opensmiles.org/opensmiles.html

pub use crate::core::Bond;
pub use crate::core::Element;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct RingBond {
    pub bond: Option<Bond>,
    pub ring_number: usize,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Symbol {
    Element(Element),
    Aromatic(Element), // TODO: Theoretically only organic subset but whatever
    Wildcard,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Atom {
    pub isotope: Option<u16>,
    pub symbol: Symbol,
    // chiral
    pub hydrogens: Option<u8>,
    pub charge: Option<i8>,
    pub atom_class: Option<usize>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct BranchedAtom {
    pub atom: Atom,
    pub ring_bonds: Vec<RingBond>,
    pub branches: Vec<Branch>,
}

/// Represents a branched atom connected to a chain via a bond.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ChainBond {
    pub bond: Option<Bond>,
    pub branched_atom: BranchedAtom,
    pub chain: Box<Chain>,
}

// FIXME: The issue is that this is now backwards. The top level of the chain will be the end of
// the branch. Therefore its going to be problematic when adding bonds. A possible solution of
// course would be to recurse through the chain prior to adding the bonds: you build up the mol
// graph of the chain first, then the last atom gets added.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Branch {
    pub bond: Option<Bond>,
    pub chain: Chain,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Chain {
    ChainBond(ChainBond),
    BranchedAtom(BranchedAtom),
}
