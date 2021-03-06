use crate::daylight::ast::*;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{digit1, satisfy},
    combinator::{map_res, opt, recognize, success, value},
    error::ParseError,
    multi::{fold_many0, many0, many_m_n},
    sequence::{pair, preceded, terminated, tuple},
    AsChar, Compare, IResult, InputIter, InputLength, InputTake, Offset, Parser, Slice,
};
use std::ops::{RangeFrom, RangeTo};

// TODO: Visibility? I want it to be seen by the test module...
// TODO: Must be a shorter way of specifying trait bounds
/// Recognizes up to n ASCII numerical characters. Fails if less than m
/// characters were recognized.
pub fn digit_m_n<T, E>(m: usize, n: usize) -> impl FnMut(T) -> IResult<T, T, E>
where
    T: Clone + PartialEq + Offset + Slice<RangeFrom<usize>> + Slice<RangeTo<usize>> + InputIter,
    <T as InputIter>::Item: AsChar,
    E: ParseError<T>,
{
    recognize(many_m_n(m, n, satisfy(|c| c.is_dec_digit())))
}

pub fn bracketed<I, T, O, E, F>(left: T, right: T, parser: F) -> impl FnMut(I) -> IResult<I, O, E>
where
    I: InputTake + Compare<T>,
    T: InputLength + Clone,
    F: Parser<I, O, E>,
    E: ParseError<I>,
{
    preceded(tag(left), terminated(parser, tag(right)))
}

pub fn hydrogens(input: &str) -> IResult<&str, u8> {
    preceded(
        tag("H"),
        alt((
            map_res(digit_m_n(1, 1), |num_str: &str| num_str.parse::<u8>()),
            success(1u8),
        )),
    )(input)
}

pub fn charge(input: &str) -> IResult<&str, i8> {
    map_res(
        pair(
            alt((value(-1, tag("-")), value(1, tag("+")))),
            alt((
                map_res(digit_m_n(1, 2), |num_str: &str| num_str.parse::<i8>()),
                success(1i8),
            )),
        ),
        |(sign, num): (i8, i8)| -> Result<i8, ()> {
            match num {
                num if num <= 15 => Ok(sign * num),
                // TODO: Proper error
                _ => Err(()),
            }
        },
    )(input)
}

// TODO: Major DRY problems
pub fn organic_symbol(input: &str) -> IResult<&str, Symbol> {
    let el = |el, t| value(Symbol::Element(el), tag(t));
    let ar = |el, t| value(Symbol::Aromatic(el), tag(t));
    alt((
        // Two letter elements must come first
        el(Element::Cl, "Cl"),
        el(Element::Br, "Br"),
        el(Element::B, "B"),
        el(Element::C, "C"),
        el(Element::N, "N"),
        el(Element::O, "O"),
        el(Element::S, "S"),
        el(Element::P, "P"),
        el(Element::F, "F"),
        el(Element::I, "I"),
        ar(Element::B, "b"),
        ar(Element::C, "c"),
        ar(Element::N, "n"),
        ar(Element::O, "o"),
        ar(Element::S, "s"),
        ar(Element::P, "p"),
        value(Symbol::Wildcard, tag("*")),
    ))(input)
}

pub fn symbol(input: &str) -> IResult<&str, Symbol> {
    let ar = |el, t| value(Symbol::Aromatic(el), tag(t));
    alt((
        map_res(
            recognize(tuple((
                satisfy(|c| c.is_ascii_uppercase()),
                opt(satisfy(|c| c.is_ascii_lowercase())),
            ))),
            |el: &str| -> Result<Symbol, ()> {
                let el: Element = el.parse()?;
                Ok(Symbol::Element(el))
            },
        ),
        ar(Element::Se, "se"),
        ar(Element::As, "as"),
        ar(Element::B, "b"),
        ar(Element::C, "c"),
        ar(Element::N, "n"),
        ar(Element::O, "o"),
        ar(Element::S, "s"),
        ar(Element::P, "p"),
        value(Symbol::Wildcard, tag("*")),
    ))(input)
}

pub fn atom_class(input: &str) -> IResult<&str, usize> {
    map_res(preceded(tag(":"), digit1), |num_str: &str| {
        num_str.parse::<usize>()
    })(input)
}

pub fn isotope(input: &str) -> IResult<&str, u16> {
    map_res(digit1, |num_str: &str| num_str.parse::<u16>())(input)
}

pub fn bracket_atom(input: &str) -> IResult<&str, Atom> {
    map_res(
        preceded(
            tag("["),
            terminated(
                tuple((
                    opt(isotope),
                    symbol,
                    // opt chiral
                    opt(hydrogens),
                    opt(charge),
                    opt(atom_class),
                )),
                tag("]"),
            ),
        ),
        |(isotope, symbol, hydrogens, charge, atom_class)| -> Result<Atom, ()> {
            Ok(Atom {
                isotope,
                symbol,
                hydrogens,
                charge,
                atom_class,
            })
        },
    )(input)
}

pub fn organic_atom(input: &str) -> IResult<&str, Atom> {
    map_res(organic_symbol, |symbol| -> Result<Atom, ()> {
        Ok(Atom {
            symbol,
            isotope: None,
            charge: None,
            atom_class: None,
            hydrogens: None,
        })
    })(input)
}

pub fn atom(input: &str) -> IResult<&str, Atom> {
    alt((bracket_atom, organic_atom))(input)
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

pub fn ring_bond(input: &str) -> IResult<&str, RingBond> {
    map_res(
        tuple((
            opt(bond),
            map_res(
                alt((digit_m_n(1, 1), preceded(tag("%"), digit_m_n(2, 2)))),
                |num_str: &str| num_str.parse::<usize>(),
            ),
        )),
        |(bond, ring_number): (Option<Bond>, usize)| -> Result<RingBond, ()> {
            Ok(RingBond { bond, ring_number })
        },
    )(input)
}

pub fn branched_atom(input: &str) -> IResult<&str, BranchedAtom> {
    map_res(
        tuple((atom, many0(ring_bond), many0(branch))),
        |(atom, ring_bonds, branches): (Atom, Vec<RingBond>, Vec<Branch>)| -> Result<BranchedAtom, ()> {
            Ok(BranchedAtom { atom, ring_bonds, branches })
        }
    )(input)
}

pub fn chain(input: &str) -> IResult<&str, Chain> {
    let (input, init) = branched_atom(input)?;

    fold_many0(
        tuple((opt(bond), branched_atom)),
        Chain::BranchedAtom(init),
        |chain: Chain, (bond, branched_atom): (Option<Bond>, BranchedAtom)| -> Chain {
            Chain::ChainBond(ChainBond {
                bond,
                branched_atom,
                chain: Box::new(chain),
            })
        },
    )(input)
}

pub fn branch(input: &str) -> IResult<&str, Branch> {
    map_res(
        bracketed("(", ")", tuple((opt(bond), chain))),
        |(bond, chain): (Option<Bond>, Chain)| -> Result<Branch, ()> { Ok(Branch { bond, chain }) },
    )(input)
}
