mod parse;

pub use parse::molecule_from_smiles;

#[cfg(test)]
mod tests {
    use crate::core::*;
    use crate::daylight::parse::*;

    #[test]
    fn parse_element() {
        assert_eq!(element("CCN"), Ok(("CN", Element::C)));
        assert_eq!(element("BrCN"), Ok(("CN", Element::Br)));
        assert_eq!(element("Og"), Ok(("", Element::Og)));
        assert_eq!(
            element("XyCN"),
            Err(nom::Err::Error(nom::error::Error {
                code: nom::error::ErrorKind::MapRes,
                input: "XyCN",
            }))
        );
        assert_eq!(
            element("X"),
            Err(nom::Err::Error(nom::error::Error {
                code: nom::error::ErrorKind::MapRes,
                input: "X",
            }))
        );
    }

    #[test]
    fn parse_bond() {
        assert_eq!(bond("-C"), Ok(("C", Bond::Single)));
        assert_eq!(bond("=C"), Ok(("C", Bond::Double)));
        assert_eq!(bond("#C"), Ok(("C", Bond::Triple)));
        assert_eq!(bond(":C"), Ok(("C", Bond::Aromatic)));
        assert_eq!(
            bond("~C"),
            Err(nom::Err::Error(nom::error::Error {
                code: nom::error::ErrorKind::Tag,
                input: "~C",
            }))
        );
    }
}
