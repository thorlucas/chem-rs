mod ast;
mod parse;

#[cfg(test)]
mod tests {
    use crate::daylight::ast::*;
    use crate::daylight::parse::*;
    use nom::{
        error::{Error, ErrorKind},
        Err, IResult,
    };

    fn ok<T>(rest: &str, res: T) -> IResult<&str, T> {
        Ok((rest, res))
    }

    fn err<T>(input: &str, code: ErrorKind) -> IResult<&str, T> {
        Err(Err::Error(Error { code, input }))
    }

    fn test_case<O>(test: impl Fn(&str) -> IResult<&str, O>) -> impl Fn(&str, IResult<&str, O>)
    where
        O: std::fmt::Debug + Eq,
    {
        move |input: &str, res: IResult<&str, O>| {
            assert_eq!(test(input), res);
        }
    }

    mod util {
        use super::*;

        #[test]
        fn can_parse_digit_m_n() {
            let test_case = |m: usize, n: usize, input: &str, res: IResult<&str, &str>| {
                assert_eq!(digit_m_n(m, n)(input), res);
            };

            test_case(2, 2, "42", ok("", "42"));
            test_case(1, 3, "42", ok("", "42"));
            test_case(1, 3, "4", ok("", "4"));
            test_case(1, 3, "426", ok("", "426"));
            test_case(1, 2, "426", ok("6", "42"));
            test_case(2, 3, "4", err("", ErrorKind::Satisfy));

            // Ensure that this doesn't consume the values if it fails
            assert_eq!(
                nom::branch::alt((digit_m_n(2, 2), nom::character::complete::digit0))("4"),
                ok("", "4")
            );
        }

        #[test]
        fn can_parse_bracketed() {
            let test_case = |left: &str, right: &str, input: &str, res: IResult<&str, &str>| {
                assert_eq!(
                    bracketed(left, right, nom::character::complete::digit1)(input)
                        as IResult<&str, &str>,
                    res
                );
            };
            //let ok = |rest: &str, out: &str| -> IResult<&str, &str> { Ok((rest, out)) };
            //let err = |input: &str, code: ErrorKind| -> IResult<&str, &str> {
            //Err(Err::Error(Error { code, input }))
            //};

            test_case("[", "]", "[1234]56", ok("56", "1234"));
            test_case("<", ">", "<1234>56", ok("56", "1234"));
            test_case("[", "]", "[123456", err("", ErrorKind::Tag));
        }
    }

    mod base {
        use super::*;

        #[test]
        fn can_parse_hydrogens() {
            let test_case = test_case(hydrogens);

            test_case("H3", ok("", 3));
            test_case("H", ok("", 1));
            test_case("H25", ok("5", 2));
            test_case("X2", err("X2", ErrorKind::Tag));
        }

        #[test]
        fn can_parse_charge() {
            let test_case = test_case(charge);

            test_case("+3", ok("", 3));
            test_case("-9", ok("", -9));
            test_case("+15", ok("", 15));
            test_case("-155", ok("5", -15));
            test_case("-", ok("", -1));
            test_case("+", ok("", 1));
            test_case("3", err("3", ErrorKind::Tag));
        }

        #[test]
        fn can_parse_organic_symbol() {
            let test_case = test_case(organic_symbol);

            test_case("C", ok("", Symbol::Element(Element::C)));
            test_case("CL", ok("L", Symbol::Element(Element::C)));
            test_case("Cl", ok("", Symbol::Element(Element::Cl)));
            test_case("n", ok("", Symbol::Aromatic(Element::N)));
            test_case("*", ok("", Symbol::Wildcard));
            test_case("as", err("as", ErrorKind::Tag));
            test_case("Au", err("Au", ErrorKind::Tag));
        }

        #[test]
        fn can_parse_symbol() {
            let test_case = test_case(symbol);

            test_case("C", ok("", Symbol::Element(Element::C)));
            test_case("CL", ok("L", Symbol::Element(Element::C)));
            test_case("Cl", ok("", Symbol::Element(Element::Cl)));
            test_case("n", ok("", Symbol::Aromatic(Element::N)));
            test_case("*", ok("", Symbol::Wildcard));
            test_case("as", ok("", Symbol::Aromatic(Element::As)));
            test_case("Au", ok("", Symbol::Element(Element::Au)));
            test_case("Xy", err("Xy", nom::error::ErrorKind::Tag));
        }

        #[test]
        fn can_parse_atom_class() {
            let test_case = test_case(atom_class);

            test_case(":42]", ok("]", 42usize));
            test_case(":]", err("]", nom::error::ErrorKind::Digit));
            test_case("42]", err("42]", nom::error::ErrorKind::Tag));
        }

        #[test]
        fn can_parse_isotope() {
            let test_case = test_case(isotope);

            test_case("12C", ok("C", 12u16));
            test_case("999Og", ok("Og", 999u16));
        }

        #[test]
        fn can_parse_bond() {
            let test_case = test_case(bond);

            test_case("-C", ok("C", Bond::Single));
            test_case("=C", ok("C", Bond::Double));
            test_case("#C", ok("C", Bond::Triple));
            test_case(":C", ok("C", Bond::Aromatic));
            test_case("~C", err("~C", ErrorKind::Tag));
        }

        #[test]
        fn can_parse_ring_bond() {
            let test_case = test_case(ring_bond);

            fn ok(rest: &str, bond: Option<Bond>, ring_number: usize) -> IResult<&str, RingBond> {
                self::ok(rest, RingBond { bond, ring_number })
            }

            test_case("=1C", ok("C", Some(Bond::Double), 1));
            test_case("9c", ok("c", None, 9));
            test_case("#24N", ok("4N", Some(Bond::Triple), 2));
            test_case("#%24N", ok("N", Some(Bond::Triple), 24));
            test_case("%245N", ok("5N", None, 24));
            test_case("=C", err("C", ErrorKind::Tag));
        }
    }

    mod atom {
        use super::*;

        fn ok(
            rest: &str,
            symbol: Symbol,
            isotope: Option<u16>,
            hydrogens: Option<u8>,
            charge: Option<i8>,
            atom_class: Option<usize>,
        ) -> IResult<&str, Atom> {
            super::ok(
                rest,
                Atom {
                    symbol,
                    isotope,
                    hydrogens,
                    charge,
                    atom_class,
                },
            )
        }

        #[test]
        fn can_parse_bracket_atom() {
            let test_case = test_case(bracket_atom);

            test_case(
                "[12CH3+:2]",
                ok(
                    "",
                    Symbol::Element(Element::C),
                    Some(12),
                    Some(3),
                    Some(1),
                    Some(2),
                ),
            );
            test_case(
                "[Br]",
                ok("", Symbol::Element(Element::Br), None, None, None, None),
            );
            test_case(
                "[nH-2:35]CC",
                ok(
                    "CC",
                    Symbol::Aromatic(Element::N),
                    None,
                    Some(1),
                    Some(-2),
                    Some(35),
                ),
            );
            test_case("[1h+:1]", err("h+:1]", ErrorKind::Tag));
            test_case("[1+:1]", err("+:1]", ErrorKind::Tag));
            test_case("[]", err("]", ErrorKind::Tag));
        }

        #[test]
        fn can_parse_organic_atom() {
            let test_case = test_case(organic_atom);

            test_case(
                "Br",
                ok("", Symbol::Element(Element::Br), None, None, None, None),
            );
            test_case(
                "OBr",
                ok("Br", Symbol::Element(Element::O), None, None, None, None),
            );
            test_case(
                "nCl",
                ok("Cl", Symbol::Aromatic(Element::N), None, None, None, None),
            );
            test_case("*C", ok("C", Symbol::Wildcard, None, None, None, None));
        }

        #[test]
        fn can_parse_atom() {
            let test_case = test_case(atom);

            test_case(
                "[12CH3+:2]CC",
                ok(
                    "CC",
                    Symbol::Element(Element::C),
                    Some(12),
                    Some(3),
                    Some(1),
                    Some(2),
                ),
            );
            test_case(
                "OBr",
                ok("Br", Symbol::Element(Element::O), None, None, None, None),
            );
        }
    }

    mod chain {
        use super::*;

        /// Simple branched atom with no branches or rings for testing purposes
        fn s_ba(element: Element) -> BranchedAtom {
            BranchedAtom {
                atom: Atom {
                    isotope: None,
                    symbol: Symbol::Element(element),
                    hydrogens: None,
                    charge: None,
                    atom_class: None,
                },
                ring_bonds: vec![],
                branches: vec![],
            }
        }

        fn bs_ba(element: Element, branches: Vec<Branch>) -> BranchedAtom {
            let mut ba = s_ba(element);
            ba.branches = branches;
            ba
        }

        /// Simple straight chain of single bonds for testing purposes
        fn s_cn(elements: &[Element]) -> Chain {
            let init = Chain::BranchedAtom(s_ba(*elements.first().expect("Empty vector")));
            a_cn(init, &elements[1..])
        }

        /// Add a simple straight chain to an existing chain for testing purposes
        fn a_cn(chain: Chain, elements: &[Element]) -> Chain {
            elements.iter().fold(chain, |chain, el| {
                Chain::ChainBond(ChainBond {
                    bond: None,
                    branched_atom: s_ba(*el),
                    chain: Box::new(chain),
                })
            })
        }

        #[test]
        fn can_parse_chain() {
            let test_case = test_case(chain);

            test_case("CC", ok("", s_cn(&[Element::C, Element::C])));
            test_case(
                "CCOC",
                ok("", s_cn(&[Element::C, Element::C, Element::O, Element::C])),
            );
            test_case(
                "N=CCO",
                ok(
                    "",
                    a_cn(
                        Chain::ChainBond(ChainBond {
                            bond: Some(Bond::Double),
                            branched_atom: s_ba(Element::C),
                            chain: Box::new(Chain::BranchedAtom(s_ba(Element::N))),
                        }),
                        &[Element::C, Element::O],
                    ),
                ),
            );
        }

        #[test]
        fn can_parse_branches() {
            let test_case = test_case(branch);

            test_case(
                "(CCO)",
                ok(
                    "",
                    Branch {
                        bond: None,
                        chain: s_cn(&[Element::C, Element::C, Element::O]),
                    },
                ),
            );
            test_case(
                "(=CN)C",
                ok(
                    "C",
                    Branch {
                        bond: Some(Bond::Double),
                        chain: s_cn(&[Element::C, Element::N]),
                    },
                ),
            );
            test_case(
                "(-CN)C",
                ok(
                    "C",
                    Branch {
                        bond: Some(Bond::Single),
                        chain: s_cn(&[Element::C, Element::N]),
                    },
                ),
            );
            test_case(
                "(-C)C",
                ok(
                    "C",
                    Branch {
                        bond: Some(Bond::Single),
                        chain: s_cn(&[Element::C]),
                    },
                ),
            );
        }

        #[test]
        fn can_parse_branched_atom() {
            let test_case = test_case(branched_atom);

            test_case(
                "C(CO)CC",
                ok(
                    "CC",
                    bs_ba(
                        Element::C,
                        vec![Branch {
                            bond: None,
                            chain: s_cn(&[Element::C, Element::O]),
                        }],
                    ),
                ),
            );
            test_case(
                "N(-CO)(CO)CC",
                ok(
                    "CC",
                    bs_ba(
                        Element::N,
                        vec![
                            Branch {
                                bond: Some(Bond::Single),
                                chain: s_cn(&[Element::C, Element::O]),
                            },
                            Branch {
                                bond: None,
                                chain: s_cn(&[Element::C, Element::O]),
                            },
                        ],
                    ),
                ),
            );

            test_case(
                "N(-C)(CC)CC",
                ok(
                    "CC",
                    bs_ba(
                        Element::N,
                        vec![
                            Branch {
                                bond: Some(Bond::Single),
                                chain: s_cn(&[Element::C]),
                            },
                            Branch {
                                bond: None,
                                chain: s_cn(&[Element::C, Element::C]),
                            },
                        ],
                    ),
                ),
            );
            test_case(
                "[N+](CC)(CC)(CC)CC",
                ok(
                    "CC",
                    BranchedAtom {
                        atom: Atom {
                            isotope: None,
                            symbol: Symbol::Element(Element::N),
                            charge: Some(1),
                            hydrogens: None,
                            atom_class: None,
                        },
                        ring_bonds: vec![],
                        branches: vec![
                            Branch {
                                bond: None,
                                chain: s_cn(&[Element::C, Element::C]),
                            },
                            Branch {
                                bond: None,
                                chain: s_cn(&[Element::C, Element::C]),
                            },
                            Branch {
                                bond: None,
                                chain: s_cn(&[Element::C, Element::C]),
                            },
                        ],
                    },
                ),
            );
        }
    }
}
