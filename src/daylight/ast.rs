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
    pub hydrogens: u8,
    pub charge: i8,
    pub atom_class: Option<usize>,
}
