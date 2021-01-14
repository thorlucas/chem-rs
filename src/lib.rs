pub mod core;
pub mod daylight;

#[cfg(test)]
mod tests {
    use crate::core::*;
    use crate::daylight::*;

    fn c() -> Atom {
        Atom {
            element: Element::C,
        }
    }

    fn n() -> Atom {
        Atom {
            element: Element::N,
        }
    }

    fn methylamine() -> (Molecule, AtomIndex, AtomIndex) {
        let mut mol = Molecule::new();
        let c = mol.add_atom(c());
        let n = mol.add_atom(n());
        mol.add_bond(c, n, Bond::Single).unwrap();

        (mol, c, n)
    }

    #[test]
    fn elements_have_atomic_number() {
        assert_eq!(Element::H.atomic_number(), 1);
        assert_eq!(Element::Tm.atomic_number(), 69);
        assert_eq!(Element::Og.atomic_number(), 118);
    }

    #[test]
    fn molecules_can_be_created() {
        // Methylamine
        let (mol, ci, ni) = methylamine();

        assert_eq!(*mol.atom(ci), c());
        assert_eq!(*mol.atom(ni), n());
    }

    #[test]
    fn molecules_have_unique_bonds() {
        // Methylamine
        let (mut mol, ci, ni) = methylamine();

        assert_eq!(mol.add_bond(ci, ni, Bond::Single), Err(()));
    }

    #[test]
    fn molecules_can_be_compared() {
        let (mola, _, _) = methylamine();
        let molb = {
            let mut mol = Molecule::new();
            // Atoms are added in reverse to have different numbers
            let n = mol.add_atom(n());
            let c = mol.add_atom(c());
            mol.add_bond(c, n, Bond::Single).unwrap();
            mol
        };
        let molc = {
            let mut mol = Molecule::new();
            // Atoms are added in reverse to have different numbers
            let n = mol.add_atom(n());
            let c1 = mol.add_atom(c());
            let c2 = mol.add_atom(c());
            mol.add_bond(c1, n, Bond::Single).unwrap();
            mol.add_bond(c2, c1, Bond::Single).unwrap();
            mol
        };
        let mold = {
            let mut mol = Molecule::new();
            // Atoms are added in reverse to have different numbers
            let n = mol.add_atom(n());
            let c1 = mol.add_atom(c());
            let c2 = mol.add_atom(c());
            mol.add_bond(c1, n, Bond::Single).unwrap();
            mol.add_bond(c2, n, Bond::Single).unwrap();
            mol
        };
        let mole = {
            let mut mol = Molecule::new();
            // Atoms are added in reverse to have different numbers
            let n = mol.add_atom(n());
            let c2 = mol.add_atom(c());
            let c1 = mol.add_atom(c());
            mol.add_bond(c2, c1, Bond::Single).unwrap();
            mol.add_bond(c1, n, Bond::Single).unwrap();
            mol
        };

        assert_eq!(mola, molb);
        assert!(mola != molc);
        assert!(molc != mold);
        assert_eq!(molc, mole);
    }

    /// Tests to parse simple straight chain SMILES
    #[test]
    fn can_parse_simple_smiles() {
        let mola = molecule_from_smiles("CN");
        let (molb, _, _) = methylamine();

        assert_eq!(mola, Ok(molb));
    }
}
