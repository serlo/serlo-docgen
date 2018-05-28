use std::collections::HashMap;
use preamble::*;

use std::io;



/// serialize to html
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(default)]
pub struct HTMLTarget {
    #[serde(skip_serializing_if = "is_default")]
    pub extension_mapping: HashMap<String, String>,
}

impl Target for HTMLTarget {
    fn do_include_sections(&self) -> bool { true }
    fn get_target_extension(&self) -> &str { "html" }
    fn get_extension_mapping(&self) -> &HashMap<String, String> {
        &self.extension_mapping
    }
    fn export<'a>(&self,
                root: &'a Element,
                settings: &Settings,
                _: &[String],
                out: &mut io::Write) -> io::Result<()> {
                    
                print!("Hello World!");
                Ok(())
        
    }
}
