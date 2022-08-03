//! Implements the `formula` target which extracts all math
//! from a document.

use crate::preamble::*;
use edtr_types::*;
use mfnf_template_spec::KnownTemplate;
use mfnf_template_spec::*;
use std::path::PathBuf;
use thiserror::Error;

use std::io;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct EdtrArgs {
    /// Title of the document beeing processed.
    document_title: String,
    //// Path to a list of link targets (anchors) available in the export.
    // #[structopt(parse(try_from_str = "load_anchor_set"))]
    // available_anchors: HashSet<String>,
}

/// Transform articles to Edtr.io state for Serlo.org
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(default)]
pub struct EdtrTarget {}

fn text_plugin_from(input: String) -> EdtrPlugin {
    EdtrPlugin::Text(vec![EdtrText::from(input)])
}

impl<'a, 's> Target<&'a EdtrArgs, &'s Settings> for EdtrTarget {
    fn target_type(&self) -> TargetType {
        TargetType::Formula
    }
    fn export(
        &self,
        root: &Element,
        _settings: &'s Settings,
        args: &'a EdtrArgs,
        out: &mut dyn io::Write,
    ) -> io::Result<()> {
        let mut builder = StateBuilder::new(args.document_title.clone());
        let mut state = builder.export(root).expect("export error");

        // serialize the root element
        serde_json::to_writer(out, &state.pop()).expect("serialization error");
        assert!(state.is_empty());
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum EdtrExportError {
    #[error("Only text state is permitted here!")]
    PluginInTextOnlyLocation,
    #[error("unknown export error")]
    Unknown,
}

type EdtrResult<T> = Result<T, EdtrExportError>;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct StateBuilder<'e> {
    pub document_title: String,

    #[serde(skip)]
    pub path: Vec<&'e Element>,
}

impl StateBuilder<'_> {
    pub fn new(document_title: String) -> Self {
        StateBuilder {
            document_title,
            path: vec![],
        }
    }
}

impl<'e> StateBuilder<'e> {
    fn export_doc_root(&mut self, doc: &Document) -> EdtrResult<EdtrArticle> {
        Ok(EdtrArticle {
            introduction: Box::new(
                EdtrArticleIntroduction {
                    explanation: Box::new(text_plugin_from(self.document_title.clone())),
                    multimedia: Box::new(EdtrPlugin::Image(EdtrImage {
                        src: "https://upload.wikimedia.org/wikipedia/commons/5/59/Titelbild_MFNF_mit_Serlo-Logo.jpg".into(),
                        alt: None,
                        caption: Box::new(text_plugin_from("dummy multimedia".into())),
                    })),
                    illustrating: true,
                    width: 50,
                }
                .into(),
            ),
            exercises: vec![PathBuf::from("/130886").into()],
            exercise_folder: EdtrArticleReference {
                id: "0".into(),
                title: "dummy".into(),
            },
            related_content: EdtrArticleRelatedContent {
                articles: vec![],
                courses: vec![],
                videos: vec![],
            },
            sources: vec![],
            content: Box::new(EdtrPlugin::Rows(self.export_vec(&doc.content)?)),
        })
    }

    /// export a vector of elements that may only be text
    fn export_text_vec(&mut self, input: &[Element]) -> EdtrResult<Vec<EdtrText>> {
        let mut result = vec![];
        for e in input {
            for node in self.export(e)? {
                match node {
                    EdtrPlugin::Text(t) => result.extend_from_slice(&t),
                    _ => return Err(EdtrExportError::PluginInTextOnlyLocation),
                };
            }
        }
        Ok(result)
    }

    fn export_heading(&mut self, heading: &Heading) -> EdtrResult<Vec<EdtrPlugin>> {
        let head = EdtrText::NestedText(EdtrMarkupText::Heading {
            level: heading.depth,
            children: self.export_text_vec(&heading.caption)?,
        });
        let mut elems = vec![vec![head].into()];
        for child in &heading.content {
            elems.extend(self.export(child)?)
        }
        Ok(elems)
    }

    // FIXME: must markup be flattened?
    fn export_formatted_text(&mut self, text: &Formatted) -> EdtrResult<Vec<EdtrPlugin>> {
        let main = match text.markup {
            MarkupType::Math => EdtrText::NestedText(EdtrMarkupText::Math {
                src: extract_plain_text(&text.content),
                inline: true,
                children: vec![EdtrText::from(extract_plain_text(&text.content))],
            }),
            _ => EdtrText::from("unimplemented markup!".to_owned()),
        };
        Ok(vec![vec![main].into()])
    }

    fn export_paragraph(&mut self, text: &Paragraph) -> EdtrResult<Vec<EdtrPlugin>> {
        Ok(vec![vec![EdtrText::NestedText(
            EdtrMarkupText::Paragraph {
                children: self.export_text_vec(&text.content)?,
            },
        )]
        .into()])
    }

    fn build_template_box(
        &mut self,
        template: &KnownTemplate<'_>,
        variant: EdtrBoxType,
    ) -> EdtrResult<EdtrPlugin> {
        let title = self.export_text_vec(template.find("title").map(|a| a.value).unwrap_or(&[]))?;

        let mut content = vec![];

        // fixme: handle anchor

        for attribute in template.present() {
            if attribute.name == "title" {
                continue;
            }

            // make sure the editor gets a consitent paragraph
            let par = Element::Paragraph(Paragraph {
                position: Span::default(),
                content: attribute.value.to_vec(),
            });
            content.extend(self.export(&par)?)
        }

        Ok(EdtrBox {
            box_type: variant,
            anchor_id: "box-1".to_owned(),
            title: Box::new(title.into()),
            content: Box::new(content.into()),
        }
        .into())
    }

    fn export_template(&mut self, template: &Template) -> EdtrResult<EdtrPlugin> {
        let parsed = if let Some(parsed) = parse_template(&template) {
            parsed
        } else {
            let pos = &template.position;
            let msg = format! {"template {} at {}:{} to {}:{} unknown or malformed!",
                &extract_plain_text(&template.name).trim().to_lowercase(), pos.start.line, pos.start.col, pos.end.line, pos.end.col
            };
            return Ok(text_plugin_from(msg));
        };

        match &parsed {
            KnownTemplate::Definition(_) => {
                self.build_template_box(&parsed, EdtrBoxType::Definition)
            }
            KnownTemplate::Theorem(_) => self.build_template_box(&parsed, EdtrBoxType::Theorem),
            KnownTemplate::Example(_) => self.build_template_box(&parsed, EdtrBoxType::Example),
            KnownTemplate::Proof(_) => self.build_template_box(&parsed, EdtrBoxType::Proof),
            KnownTemplate::Warning(_) => self.build_template_box(&parsed, EdtrBoxType::Attention),
            KnownTemplate::Hint(_) => self.build_template_box(&parsed, EdtrBoxType::Note),
            KnownTemplate::SolutionProcess(_) => {
                self.build_template_box(&parsed, EdtrBoxType::Approach)
            }
            _ => Ok(text_plugin_from(
                format! {"unimplemented plugin: {}", extract_plain_text(&template.name)},
            )),
        }
    }

    fn export_vec(&mut self, elems: &[Element]) -> EdtrResult<Vec<EdtrPlugin>> {
        let mut result = vec![];
        for e in elems {
            result.extend(self.export(e)?)
        }
        Ok(result)
    }

    pub fn export(&mut self, node: &Element) -> EdtrResult<Vec<EdtrPlugin>> {
        Ok(match node {
            Element::Document(ref doc) => vec![self.export_doc_root(doc)?.into()],
            Element::Heading(ref heading) => self.export_heading(heading)?,
            Element::Text(Text { text, .. }) => vec![text_plugin_from(text.clone())],
            Element::Formatted(formatted) => self.export_formatted_text(formatted)?,
            Element::Paragraph(par) => self.export_paragraph(par)?,
            Element::Template(template) => vec![self.export_template(template)?],
            _ => vec![text_plugin_from("unimplemented!".into())],
        })
    }
}
