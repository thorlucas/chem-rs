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

    //#[test]
    //fn parse_element() {
    //assert_eq!(element("CCN"), Ok(("CN", Element::C)));
    //assert_eq!(element("BrCN"), Ok(("CN", Element::Br)));
    //assert_eq!(element("Og"), Ok(("", Element::Og)));
    //assert_eq!(
    //element("XyCN"),
    //Err(nom::Err::Error(nom::error::Error {
    //code: nom::error::ErrorKind::MapRes,
    //input: "XyCN",
    //}))
    //);
    //assert_eq!(
    //element("X"),
    //Err(nom::Err::Error(nom::error::Error {
    //code: nom::error::ErrorKind::MapRes,
    //input: "X",
    //}))
    //);
    //}

    //#[test]
    //fn parse_bond() {
    //assert_eq!(bond("-C"), Ok(("C", Bond::Single)));
    //assert_eq!(bond("=C"), Ok(("C", Bond::Double)));
    //assert_eq!(bond("#C"), Ok(("C", Bond::Triple)));
    //assert_eq!(bond(":C"), Ok(("C", Bond::Aromatic)));
    //assert_eq!(
    //bond("~C"),
    //Err(nom::Err::Error(nom::error::Error {
    //code: nom::error::ErrorKind::Tag,
    //input: "~C",
    //}))
    //);
    //}

    fn ok<T>(rest: &str, res: T) -> IResult<&str, T> {
        Ok((rest, res))
    }

    fn err<T>(input: &str, code: ErrorKind) -> IResult<&str, T> {
        Err(Err::Error(Error { code, input }))
    }

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

    #[test]
    fn can_parse_hydrogens() {
        let test_case = |input: &str, res: IResult<&str, u8>| {
            assert_eq!(hydrogens(input), res);
        };

        test_case("H3", ok("", 3));
        test_case("H", ok("", 1));
        test_case("H25", ok("5", 2));
        test_case("X2", err("X2", ErrorKind::Tag));
    }

    #[test]
    fn can_parse_charge() {
        let test_case = |input: &str, res: IResult<&str, i8>| {
            assert_eq!(charge(input), res);
        };

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
        let test_case = |input: &str, res: IResult<&str, Symbol>| {
            assert_eq!(organic_symbol(input), res);
        };

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
        let test_case = |input: &str, res: IResult<&str, Symbol>| {
            assert_eq!(symbol(input), res);
        };

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
        let test_case = |input: &str, res: IResult<&str, usize>| {
            assert_eq!(atom_class(input), res);
        };

        test_case(":42]", ok("]", 42usize));
        test_case(":]", err("]", nom::error::ErrorKind::Digit));
        test_case("42]", err("42]", nom::error::ErrorKind::Tag));
    }

    #[test]
    fn can_parse_isotope() {
        let test_case = |input: &str, res: IResult<&str, u16>| {
            assert_eq!(isotope(input), res);
        };

        test_case("12C", ok("C", 12u16));
        test_case("999Og", ok("Og", 999u16));
    }

    #[test]
    fn can_parse_bracket_atom() {
        let test_case = |input: &str, res: IResult<&str, Atom>| {
            assert_eq!(bracket_atom(input), res);
        };

        fn ok(
            rest: &str,
            symbol: Symbol,
            isotope: Option<u16>,
            hydrogens: u8,
            charge: i8,
            atom_class: Option<usize>,
        ) -> IResult<&str, Atom> {
            self::ok(
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

        test_case(
            "[12CH3+:2]",
            ok("", Symbol::Element(Element::C), Some(12), 3, 1, Some(2)),
        );
        test_case(
            "[Br]",
            ok("", Symbol::Element(Element::Br), None, 0, 0, None),
        );
        test_case(
            "[nH-2:35]CC",
            ok("CC", Symbol::Aromatic(Element::N), None, 1, -2, Some(35)),
        );
        test_case("[1h+:1]", err("h+:1]", ErrorKind::Tag));
        test_case("[1+:1]", err("+:1]", ErrorKind::Tag));
        test_case("[]", err("]", ErrorKind::Tag));
    }

    #[test]
    fn can_parse_bond() {
        let test_case = |input: &str, res: IResult<&str, Bond>| {
            assert_eq!(bond(input), res);
        };

        test_case("-C", ok("C", Bond::Single));
        test_case("=C", ok("C", Bond::Double));
        test_case("#C", ok("C", Bond::Triple));
        test_case(":C", ok("C", Bond::Aromatic));
        test_case("~C", err("~C", ErrorKind::Tag));
    }

    #[test]
    fn can_parse_ring_bond() {
        let test_case = |input: &str, res: IResult<&str, RingBond>| {
            assert_eq!(ring_bond(input), res);
        };

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
