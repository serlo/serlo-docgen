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
pub struct TemplateSpec<'p, ID> {
    pub id: ID,
    pub names: Vec<String>,
    pub format: Format,
    pub attributes: Vec<Attribute<'p>>,
}

/// Represents an attribute (or argument) of a template.
#[derive(Clone, Serialize)]
pub struct Attribute<'p> {
    pub names: Vec<String>,
    pub priority: Priority,
    #[serde(skip)]
    pub predicate: &'p Predicate,
    pub predicate_source: String,
}

impl<'p> fmt::Debug for Attribute<'p> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Attribute: {{ names {:?}, \
                   priority: {:?}, predicate: <predicate func>, \
                   predicate_source: {:?} }}", self.names,
                   self.priority, self.predicate_source)
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
            id: $id:ident,
            names: [$($name:expr),*],
            format: $format:expr,
            attributes: [$(
                {
                    ident: $attr_id:ident,
                    names: [$($attr_name:expr),*],
                    priority: $priority:expr,
                    predicate: $predicate:expr
                }
            ),*]
        }
    ),*) => {
        /// The templates availabe.
        #[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
        pub enum TemplateID {
            $(
                $id
            ),*
        }

        /// Full template information.
        #[derive(Debug, Clone, PartialEq, Serialize)]
        pub enum Template<'e> {
            $(
                $id {
                    id: TemplateID,
                    names: Vec<String>,
                    format: Format,
                    $(
                        $attr_id: Option<&'e [Element]>
                    ),*
                }
            ),*
        }

        impl<'e> Template<'e> {
            pub fn id(&self) -> &TemplateID {
                match *self {
                    $(
                        Template::$id { ref id, .. } => id
                    ),*
                }
            }
            pub fn format(&self) -> &Format {
                match *self {
                    $(
                        Template::$id { ref format, .. } => format
                    ),*
                }
            }
            pub fn names(&self) -> &[String] {
                match *self {
                    $(
                        Template::$id { ref names, .. } => names
                    ),*
                }
            }
        }

        pub fn parse_template<'e>(elem: &'e Element) -> Option<Template<'e>> {
            if let Element::Template {
                ref name,
                ref content,
                ..
            } = *elem {
                let name = extract_plain_text(&name).trim().to_lowercase();
                $(
                    let names = [$($name.trim().to_lowercase()),*];
                    if names.contains(&name) {
                        return Some(Template::$id {
                            id: TemplateID::$id,
                            $(
                                $attr_id: if let Some(arg) = find_arg(content, &names) {
                                    if let Element::TemplateArgument {
                                        ref value,
                                        ..
                                    } = *arg {
                                        Some(value)
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            ),*,
                            names: names.to_vec(),
                            format: $format,
                        })
                    }
                )*
            }
            None
        }

        pub fn spec<'p>() -> Vec<TemplateSpec<'p, TemplateID>> {
            vec![
                $(
                    TemplateSpec {
                        id: TemplateID::$id,
                        names: vec![$($name.trim().to_lowercase()),*],
                        format: $format,
                        attributes: vec![$(
                            Attribute {
                                names: vec![$($attr_name.trim().to_lowercase()),*],
                                priority: $priority,
                                predicate: $predicate,
                                predicate_source: stringify!($predicate).into()
                            }
                        ),*]
                    }
                ),*
            ]
        }

        pub fn spec_of<'p>(name: &str) -> Option<TemplateSpec<'p, TemplateID>> {
            let name = name.trim().to_lowercase();
            for spec in spec() {
                if spec.names.contains(&name) {
                    return Some(spec)
                }
            }
            None
        }
    }
}
