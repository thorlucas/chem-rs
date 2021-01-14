use crate::core::*;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::satisfy,
    combinator::{complete, map_res, opt, recognize, success, value},
    multi::{fold_many0, many0},
    sequence::{pair, tuple},
    IResult,
};

#[derive(Clone, Debug)]
pub enum BondNode {
    Leaf(Element),
    Branch(Bond, Element, Box<BondNode>),
}

pub fn molecule_from_smiles(smiles: &str) -> Result<Molecule, ()> {
    match complete(molecule)(smiles) {
        Ok((_, molecule)) => Ok(molecule),
        Err(_) => Err(()),
    }
}

pub fn molecule(input: &str) -> IResult<&str, Molecule> {
    let (input, init) = element(input)?;

    let (input, tree) = fold_many0(
        alt((pair(bond, element), pair(success(Bond::Single), element))),
        Box::new(BondNode::Leaf(init)),
        |tree: Box<BondNode>, (bond, element): (Bond, Element)| {
            Box::new(BondNode::Branch(bond, element, tree))
        },
    )(input)?;

    // TODO: Looks like I'm doing this sort of wrong. Lots of repitition.
    fn collapse(molecule: &mut Molecule, last: AtomIndex, last_bond: Bond, node: Box<BondNode>) {
        match *node {
            BondNode::Branch(bond, element, next) => {
                let index = molecule.add_atom(Atom { element });
                // TODO: Error handle
                molecule
                    .add_bond(last, index, last_bond)
                    .expect("Couldn't add bond");
                collapse(molecule, index, bond, next)
            }
            BondNode::Leaf(element) => {
                // TODO: DRY?
                let index = molecule.add_atom(Atom { element });
                molecule
                    .add_bond(last, index, last_bond)
                    .expect("Couldn't add bond");
            }
        }
    }

    let mut molecule = Molecule::new();
    match *tree {
        BondNode::Branch(bond, element, next) => {
            let index = molecule.add_atom(Atom { element });
            collapse(&mut molecule, index, bond, next);
        }
        BondNode::Leaf(element) => {
            molecule.add_atom(Atom { element });
        }
    }

    Ok((input, molecule))
}

pub fn element(input: &str) -> IResult<&str, Element> {
    map_res(
        recognize(tuple((
            satisfy(|c| c.is_ascii_uppercase()),
            opt(satisfy(|c| c.is_ascii_lowercase())),
        ))),
        |el: &str| el.parse(),
    )(input)
}

pub fn bond(input: &str) -> IResult<&str, Bond> {
    let tag_bond = |t, b| value(b, tag(t));
    alt((
        tag_bond("-", Bond::Single),
        tag_bond("=", Bond::Double),
        tag_bond("#", Bond::Triple),
        tag_bond(":", Bond::Aromatic),
    ))(input)
}
