//! Implements the `formula` target which extracts all math
//! from a document.

use crate::preamble::*;
use base64;
use serde_json;
use std::collections::{HashMap, HashSet};
use std::io;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct FormulaArgs {}

/// Dump formulae to stdout
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(default)]
pub struct FormulaTarget {}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
struct FormulaCollector<'e> {
    #[serde(skip)]
    pub path: Vec<&'e Element>,

    /// set of formulae (math expressions) in the document
    pub formulae: HashSet<String>,
}

impl<'e, 's: 'e, 'a> Traversion<'e, ((), ())> for FormulaCollector<'e> {
    path_methods!('e);

    fn work(&mut self, root: &Element, _: ((), ()), _out: &mut io::Write) -> io::Result<bool> {
        match root {
            Element::Formatted(ref formatted) if formatted.markup == MarkupType::Math => {
                self.formulae.insert(extract_plain_text(&formatted.content));
            }
            _ => (),
        };
        Ok(true)
    }
}

impl<'a, 's> Target<&'a FormulaArgs, &'s Settings> for FormulaTarget {
    fn target_type(&self) -> TargetType {
        TargetType::Formula
    }
    fn export(
        &self,
        root: &Element,
        settings: &'s Settings,
        args: &'a FormulaArgs,
        out: &mut io::Write,
    ) -> io::Result<()> {
        let mut collector = FormulaCollector::default();
        collector.run(root, ((), ()), out)?;

        for formula in collector.formulae {
            writeln!(out, "{}", base64::encode(&formula))?;
        }
        Ok(())
    }
}
