//! various utility functions.

use std::io;
use std::io::Read;
use std::fs;
use std::path::{Path};
use syn::{Attribute, DeriveInput, Ident, Lit, Meta};

pub fn parse_derive(ast: DeriveInput) -> (Ident, String) {
    let name = ast.ident;

    let grammar: Vec<&Attribute> = ast.attrs
        .iter()
        .filter(|attr| match attr.interpret_meta() {
            Some(Meta::NameValue(name_value)) =>
                name_value.ident.to_string() == "spec",
            _ => false
        })
        .collect();

    let filename = match grammar.len() {
        0 => panic!("a spec file needs to be provided with \
                    the #[spec = \"...\"] attribute"),
        1 => get_filename(grammar[0]),
        _ => panic!("only 1 grammar file can be provided")
    };

    (name, filename)
}

pub fn get_filename(attr: &Attribute) -> String {
    match attr.interpret_meta() {
        Some(Meta::NameValue(name_value)) => match name_value.lit {
            Lit::Str(filename) => filename.value(),
            _ => panic!("spec attribute must be a string")
        },
        _ => panic!("spec attribute must be of the form `grammar = \"...\"`")
    }
}

pub fn read_file<P: AsRef<Path>>(path: P) -> io::Result<String> {
    let mut file = fs::File::open(path.as_ref())?;
    let mut string = String::new();
    file.read_to_string(&mut string)?;
    Ok(string)
}
