use mediawiki_parser::*;
use std::io;
use std::fmt;

/// Specifies wether a template represents a logical unit (`Block`)
/// or simpler markup (`Inline`).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Format {
    Block,
    Inline
}

/// Template attributes can have different priorities.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Priority {
    Required,
    Optional
}

/// A function to determine wether a given element is allowed.
type Predicate = Fn(&[Element]) -> bool;

/// Represents a (semantic) template.
#[derive(Debug, Clone, Serialize)]
pub struct TemplateSpec<'p> {
    pub name: String,
    pub alternative_names: Vec<String>,
    pub format: Format,
    pub attributes: Vec<Attribute<'p>>,
}

/// Represents a template instance matching with a template listed in the spec.
#[derive(Debug, Clone, Serialize)]
pub struct TemplateInstance<'e> {
    pub name: String,
    pub format: Format,
    pub attributes: Vec<AttributeInstance<'e>>,
}

/// A template attribute instance.
#[derive(Debug, Clone, Serialize)]
pub struct AttributeInstance<'e> {
    pub name: String,
    pub priority: Priority,
    pub content: &'e Element,
}

/// Represents an attribute (or argument) of a template.
#[derive(Clone, Serialize)]
pub struct Attribute<'p> {
    pub name: String,
    pub alternative_names: Vec<String>,
    pub priority: Priority,
    #[serde(skip)]
    pub predicate: &'p Predicate,
    pub predicate_source: String,
}

impl<'p> fmt::Debug for Attribute<'p> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Attribute: {{ name: {:?}, alternative_names {:?}, \
                   priority: {:?}, predicate: <predicate func>, \
                   predicate_source: {:?} }}", self.name, self.alternative_names,
                   self.priority, self.predicate_source)
    }
}

impl<'e> TemplateInstance<'e> {
    pub fn get(&self, attribute_name: &str) -> Option<&AttributeInstance> {
        for attr in &self.attributes {
            if attr.name == attribute_name {
                return Some(attr)
            }
        }
        None
    }

    pub fn get_content(&self, attribute_name: &str) -> Option<&[Element]> {
        if let Some(attr) = self.get(attribute_name) {
            if let Element::TemplateArgument { ref value, .. } = *attr.content {
                Some(value)
            } else {
                None
            }
        } else {
            None
        }
    }
}

/// Checks a predicate for a given input tree.
#[derive(Default)]
pub struct TreeChecker<'path> {
    pub path: Vec<&'path Element>,
    pub result: bool,
}

#[derive(Clone, Copy)]
enum CheckerMode {
    All,
    None,
}

struct CheckerSettings<'p> {
    pub predicate: &'p Predicate,
    pub mode: CheckerMode,
}

impl <'e, 'p: 'e> Traversion<'e, &'p CheckerSettings<'p>> for TreeChecker<'e> {

    path_methods!('e);

    fn work_vec(
        &mut self,
        root: &[Element],
        settings: &'p CheckerSettings<'p>,
        _: &mut io::Write
    ) -> io::Result<bool> {
        match settings.mode {
            CheckerMode::All => self.result &= (settings.predicate)(root),
            CheckerMode::None => self.result &= !(settings.predicate)(root),
        }
        Ok(true)
    }
}

impl<'p> TreeChecker<'p> {
    pub fn all(root: &[Element], predicate: &Predicate) -> bool {
        let settings = CheckerSettings {
            predicate,
            mode: CheckerMode::All
        };
        let mut checker = TreeChecker::default();
        checker.result = true;
        checker.run_vec(&root, &settings, &mut vec![])
            .expect("error checking predicate!");
        checker.result
    }

    pub fn min_one(root: &[Element], predicate: &Predicate) -> bool {
        !TreeChecker::never(root, predicate)
    }

    pub fn never(root: &[Element], predicate: &Predicate) -> bool {
        let settings = CheckerSettings {
            predicate,
            mode: CheckerMode::None
        };
        let mut checker = TreeChecker::default();
        checker.result = true;
        checker.run_vec(&root, &settings, &mut vec![])
            .expect("error checking predicate!");
        checker.result
    }
}

pub fn check_name(name: &[Element]) -> Option<&str> {
    if name.len() != 1 {
        return None
    }
    match name.first() {
        Some(&Element::Text { ref text, .. }) => return Some(text.trim()),
        Some(&Element::Paragraph { ref content, .. }) => {
            if content.len() != 1 {
                return None
            }
            if let Some(&Element::Text { ref text, .. }) = content.first() {
                return Some(text.trim())
            }
        },
        _ => (),
    };
    None
}

macro_rules! template_spec {
    ($(
        template {
            name: $name:expr,
            alt: [$($altname:expr),*],
            format: $format:expr,
            attributes: [$($attr:expr),*]
        }
    ),*) => {

        pub fn parse_template<'e>(elem: &'e Element) -> Option<TemplateInstance<'e>> {
            if let Element::Template {
                ref name,
                ref content,
                ..
            } = *elem {
                let name = extract_plain_text(&name);
                let spec = if let Some(spec) = spec_of(&name) {
                    spec
                } else {
                    return None
                };

                let mut args = vec![];
                for attr in spec.attributes {
                    let arg = find_arg(content, &attr.name);
                    if arg.is_none() && attr.priority == Priority::Required {
                        return None
                    }
                    if let Some(arg) = arg {
                        args.push(AttributeInstance {
                            name: attr.name.trim().to_lowercase(),
                            priority: attr.priority,
                            content: arg
                        });
                    }
                }
                return Some(TemplateInstance {
                    name: spec.name.trim().to_lowercase(),
                    format: spec.format,
                    attributes: args
                });
            }
            None
        }

        pub fn spec<'p>() -> Vec<TemplateSpec<'p>> {
            vec![
                $(
                    TemplateSpec {
                        name: $name.trim().into(),
                        alternative_names: vec![$($altname.trim().into()),*],
                        format: $format,
                        attributes: vec![$($attr),*]
                    }
                ),*
            ]
        }

        pub fn spec_of(name: &str) -> Option<TemplateSpec> {
            let name = name.trim().to_lowercase();
            $(
                let mut names = vec![$($altname.trim().to_lowercase()),*];
                names.insert(0, $name.trim().to_lowercase());

                if names.contains(&name) {
                    return Some(
                        TemplateSpec {
                            name: $name.trim().into(),
                            alternative_names: vec![$($altname.trim().into()),*],
                            format: $format,
                            attributes: vec![$($attr),*]
                        }
                    );
                }
            )*
            None
        }
    }
}

macro_rules! attribute {
    (
        name: $name:expr,
        alt: [$($altname:expr),*],
        priority: $priority:expr,
        predicate: $predicate:expr
    ) => {
        Attribute {
            name: $name.trim().into(),
            alternative_names: vec![$($altname.trim().into()),*],
            priority: $priority,
            predicate: $predicate,
            predicate_source: stringify!($predicate).into()
        }
    }
}
