//! Implements the `formula` target which extracts all math
//! from a document.

use crate::preamble::*;
use edtr_types::*;
use mfnf_template_spec::KnownTemplate;
use mfnf_template_spec::*;
use std::path::PathBuf;
use thiserror::Error;

use regex::Regex;
use std::collections::HashMap;
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

fn wrap_paragraph(input: Vec<EdtrText>) -> EdtrText {
    EdtrText::NestedText(EdtrMarkupText::Paragraph { children: input })
}

impl<'a, 's> Target<&'a EdtrArgs, &'s Settings> for EdtrTarget {
    fn target_type(&self) -> TargetType {
        TargetType::Edtr
    }
    fn export(
        &self,
        root: &Element,
        settings: &'s Settings,
        args: &'a EdtrArgs,
        out: &mut dyn io::Write,
    ) -> io::Result<()> {
        let mut builder = StateBuilder::new(args.document_title.clone(), settings);
        let mut state = builder.export(root).expect("export error");

        // serialize the root element
        serde_json::to_writer(out, &state.pop()).expect("serialization error");
        assert!(state.is_empty());
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum EdtrExportError {
    #[error("Only text state is permitted here! {0:?}")]
    PluginInTextOnlyLocation(Span),
    #[error("unknown export error")]
    Unknown,
}

type EdtrResult<T> = Result<T, EdtrExportError>;

#[derive(Debug, Clone)]
struct StateBuilder<'s> {
    pub document_title: String,

    pub settings: &'s Settings,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct TextFlags {
    em: bool,
    strong: bool,
    code: bool,
    color: usize,
}

impl<'s> StateBuilder<'s> {
    pub fn new(document_title: String, settings: &'s Settings) -> Self {
        StateBuilder {
            document_title,
            settings,
        }
    }
}

impl<'s> StateBuilder<'s> {
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

    fn propagate_text_flags(&self, input: &mut [EdtrText], flags: &TextFlags) {
        for t in input.iter_mut() {
            match t {
                EdtrText::SimpleText {
                    ref mut strong,
                    ref mut em,
                    ref mut code,
                    ref mut color,
                    ..
                } => {
                    *strong = *strong || flags.strong;
                    *em = *em || flags.em;
                    *code = *code || flags.code;
                    if flags.color != 0 {
                        *color = flags.color
                    }
                }
                EdtrText::NestedText(EdtrMarkupText::Heading {
                    ref mut children, ..
                })
                | EdtrText::NestedText(EdtrMarkupText::Paragraph { ref mut children })
                | EdtrText::NestedText(EdtrMarkupText::UnorderedList { ref mut children })
                | EdtrText::NestedText(EdtrMarkupText::OrderedList { ref mut children })
                | EdtrText::NestedText(EdtrMarkupText::ListItem { ref mut children })
                | EdtrText::NestedText(EdtrMarkupText::ListItemChild { ref mut children })
                | EdtrText::NestedText(EdtrMarkupText::Hyperlink {
                    ref mut children, ..
                })
                | EdtrText::NestedText(EdtrMarkupText::Math {
                    ref mut children, ..
                }) => self.propagate_text_flags(children, flags),
                EdtrText::Empty {} => (),
            }
        }
    }
    /// export a vector of elements that may only be text
    fn export_text_vec(&mut self, input: &[Element]) -> EdtrResult<Vec<EdtrText>> {
        let mut result = vec![];
        for e in input {
            for node in self.export(e)? {
                match node {
                    EdtrPlugin::Text(t) => result.extend_from_slice(&t),
                    _ => {
                        return Err(EdtrExportError::PluginInTextOnlyLocation(
                            e.get_position().clone(),
                        ))
                    }
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

    fn resolve_math_macros(latex: String) -> String {
        let replacements = HashMap::from([(r"\\sgn(?-u)\b", r"\mathrm{sgn}"),
                                          (r"\\Q(?-u)\b", r"\mathbb{Q}"),
                                          (r"\\begin\{align\}", r"\begin{aligned}"),
                                          (r"\\end\{align\}", r"\end{aligned}")]);
        let mut res = latex;
        for (r#macro, replacement) in &replacements {
            let re = Regex::new(r#macro).unwrap();
            res = re.replace_all(&res, &**replacement).to_string();
        }
        res
    }

    fn math_from_string(latex: String, inline: bool) -> EdtrPlugin {
        EdtrPlugin::Text(vec![EdtrText::NestedText(EdtrMarkupText::Math {
            src: Self::resolve_math_macros(latex),
            inline,
            children: vec![],
        })])
    }

    fn export_formatted_text(&mut self, text: &Formatted) -> EdtrResult<Vec<EdtrPlugin>> {
        let main = match text.markup {
            MarkupType::Math => Self::math_from_string(extract_plain_text(&text.content), true),
            MarkupType::Italic => {
                let mut content = self.export_text_vec(&text.content)?;
                let flags = TextFlags {
                    em: true,
                    strong: false,
                    code: false,
                    color: 0,
                };
                self.propagate_text_flags(&mut content, &flags);
                content.into()
            }
            MarkupType::Bold => {
                let mut content = self.export_text_vec(&text.content)?;
                let flags = TextFlags {
                    em: false,
                    strong: true,
                    code: false,
                    color: 0,
                };
                self.propagate_text_flags(&mut content, &flags);
                content.into()
            }
            MarkupType::Code | MarkupType::Preformatted | MarkupType::NoWiki => {
                let mut content = self.export_text_vec(&text.content)?;
                let flags = TextFlags {
                    em: false,
                    strong: true,
                    code: false,
                    color: 0,
                };
                self.propagate_text_flags(&mut content, &flags);
                content.into()
            }

            _ => self.make_error_box("unimplemented markup!".to_owned(), true),
        };
        Ok(vec![main])
    }

    fn export_paragraph(&mut self, text: &Paragraph) -> EdtrResult<Vec<EdtrPlugin>> {
        Ok(vec![EdtrPlugin::Text(vec![wrap_paragraph(
            self.export_text_vec(&text.content)?,
        )])])
    }

    fn make_error_box(&self, err: String, inline: bool) -> EdtrPlugin {
        let mut text = EdtrText::from(err);
        match text {
            EdtrText::SimpleText { ref mut color, .. } => *color = 2,
            _ => (),
        };
        if inline {
            vec![text].into()
        } else {
            vec![wrap_paragraph(vec![text])].into()
        }

        //EdtrBox {
        //    box_type: EdtrBoxType::Attention,
        //    anchor_id: "box-1".to_owned(),
        //    title: Box::new(text_plugin_from("Export Error".into())),
        //    content: Box::new(EdtrPlugin::Rows(vec![vec![wrap_paragraph(vec![
        //        err.into()
        //    ])]
        //    .into()])),
        //}
        //.into()
    }

    fn find_box_variant(template_name: &str, attrbute_name: &str) -> Option<EdtrBoxType> {
        match attrbute_name {
            "definition" => Some(EdtrBoxType::Definition),
            "theorem" => Some(EdtrBoxType::Theorem),
            "explanation" => Some(EdtrBoxType::Note),
            "example" => Some(EdtrBoxType::Example),
            "solutionprocess" => Some(EdtrBoxType::Approach),
            "summary" => Some(EdtrBoxType::Remember),
            "proof" => Some(EdtrBoxType::Proof),
            "proof2" => Some(EdtrBoxType::Proof),
            "hint" => Some(EdtrBoxType::Note),
            "warning" => Some(EdtrBoxType::Attention),
            "content" => match template_name {
                "important" => Some(EdtrBoxType::Quote),
                _ => None,
            },
            _ => None,
        }
    }

    fn build_template_box(&mut self, template: &KnownTemplate<'_>) -> EdtrResult<Vec<EdtrPlugin>> {
        let title = self.export_text_vec(template.find("title").map(|a| a.value).unwrap_or(&[]))?;

        // fixme: handle anchor

        let mut boxes = vec![];
        for attribute in template.present() {
            if attribute.name == "title" {
                continue;
            }

            // make sure the editor gets a consitent paragraph
            let par = Element::Paragraph(Paragraph {
                position: Span::default(),
                content: attribute.value.to_vec(),
            });
            boxes.push(
                EdtrBox {
                    box_type: Self::find_box_variant(&template.names()[0], &attribute.name)
                        .unwrap_or(EdtrBoxType::Blank),
                    anchor_id: "box26447".to_owned(),
                    title: Box::new(vec![wrap_paragraph(title.clone())].into()),
                    content: Box::new(EdtrPlugin::Rows(vec![self.export(&par)?.into()])),
                }
                .into(),
            );
        }

        Ok(boxes)
    }

    fn export_template(&mut self, template: &Template) -> EdtrResult<Vec<EdtrPlugin>> {
        let parsed = if let Some(parsed) = parse_template(&template) {
            parsed
        } else {
            let pos = &template.position;
            let msg = format! {"template {} at {}:{} to {}:{} unknown or malformed!",
                &extract_plain_text(&template.name).trim().to_lowercase(), pos.start.line, pos.start.col, pos.end.line, pos.end.col
            };
            return Ok(vec![text_plugin_from(msg)]);
        };

        match &parsed {
            KnownTemplate::Definition(_)
            | KnownTemplate::Theorem(_)
            | KnownTemplate::Example(_)
            | KnownTemplate::Proof(_)
            | KnownTemplate::Warning(_)
            | KnownTemplate::Hint(_)
            | KnownTemplate::Important(_)
            | KnownTemplate::SolutionProcess(_) => self.build_template_box(&parsed),
            KnownTemplate::Anchor(_) => {
                Ok(vec![self.make_error_box(
                    "anchor not implemented, yet".to_owned(),
                    true,
                )])
            }
            KnownTemplate::Smiley(_) => {
                Ok(vec![self.make_error_box(
                    "smiley not implemented, yet".to_owned(),
                    true,
                )])
            }
            KnownTemplate::Todo(_) => {
                Ok(vec![self.make_error_box(
                    "todo not implemented, yet".to_owned(),
                    false,
                )])
            }
            KnownTemplate::Formula(formula) => match formula.formula {
                [Element::Formatted(ref root)] => {
                    if MarkupType::Math == root.markup {
                        let formula = extract_plain_text(&root.content);
                        Ok(vec![Self::math_from_string(formula, false)])
                    } else {
                        Ok(vec![
                            self.make_error_box("malformed formula".to_owned(), false)
                        ])
                    }
                }
                _ => Ok(vec![
                    self.make_error_box("malformed formula".to_owned(), false)
                ]),
            },
            KnownTemplate::NoPrint(_) => {
                Ok(vec![self.make_error_box(
                    "noprint not implemented, yet".to_owned(),
                    false,
                )])
            }
            _ => Ok(vec![self.make_error_box(
                format! {"unimplemented plugin: {}", extract_plain_text(&template.name)},
                false,
            )]),
        }
    }

    fn export_vec(&mut self, elems: &[Element]) -> EdtrResult<Vec<EdtrPlugin>> {
        let mut result = vec![];
        for e in elems {
            result.extend(self.export(e)?)
        }
        Ok(result)
    }

    fn export_list(&mut self, list: &List) -> EdtrResult<EdtrPlugin> {
        let content = self.export_text_vec(&list.content)?;
        let item_kinds: Vec<_> = list
            .content
            .iter()
            .filter_map(|item| match item {
                Element::ListItem(ListItem { kind, .. }) => Some(*kind),
                _ => None,
            })
            .collect();
        if item_kinds.contains(&ListItemKind::Definition) {
            return Ok(text_plugin_from(
                "definition lists not implemented, yet!".to_owned(),
            ));
        }
        if item_kinds.contains(&ListItemKind::Ordered) {
            Ok(vec![EdtrText::NestedText(EdtrMarkupText::OrderedList {
                children: content,
            })]
            .into())
        } else {
            Ok(vec![EdtrText::NestedText(EdtrMarkupText::UnorderedList {
                children: content,
            })]
            .into())
        }
    }

    fn export_list_item(&mut self, item: &ListItem) -> EdtrResult<EdtrPlugin> {
        Ok(vec![EdtrText::NestedText(EdtrMarkupText::ListItem {
            children: vec![EdtrText::NestedText(EdtrMarkupText::ListItemChild {
                children: self.export_text_vec(&item.content)?,
            })],
        })]
        .into())
    }

    fn export_htmltag(&mut self, tag: &HtmlTag) -> EdtrResult<Vec<EdtrPlugin>> {
        let mut content = self.export_text_vec(&tag.content)?;
        match tag.name.to_lowercase().trim() {
            "dfn" => {
                let flags = TextFlags {
                    em: true,
                    strong: false,
                    code: false,
                    color: 0,
                };
                self.propagate_text_flags(&mut content, &flags);
                Ok(vec![content.into()])
            }
            "ref" => {
                let msg = "references / sources not supported, yet!".to_owned();
                let err = EdtrPlugin::Text(vec![msg.into()]);
                Ok(vec![err])
            }
            "section" => Ok(vec![]),
            _ => {
                let msg = format!(
                    "no export function defined \
                     for html tag `{}`!",
                    tag.name
                );
                let err = EdtrPlugin::Text(vec![msg.into()]);
                Ok(vec![err])
            }
        }
    }

    pub fn export_internalref(&mut self, iref: &InternalReference) -> EdtrResult<Vec<EdtrPlugin>> {
        let target_str = extract_plain_text(&iref.target);

        // embedded files (images, videos, ...)
        if is_file(iref, self.settings) {
            let image_path = format!(
                "https://www.mediawiki.org/w/index.php?title=Special:Redirect/file/{}",
                target_str
            );

            // collect image options
            let mut image_options = vec![];
            for option in &iref.options {
                image_options.push(extract_plain_text(option).trim().to_string());
            }

            let cap_content = wrap_paragraph(self.export_text_vec(&iref.caption)?);

            if is_centered(iref) || is_thumb(iref) {
                let image = EdtrImage {
                    src: image_path,
                    alt: Some(extract_plain_text(&iref.caption)),
                    caption: Box::new(EdtrPlugin::Text(vec![cap_content])),
                };
                Ok(vec![EdtrPlugin::Image(image)])
            // inline images
            } else {
                Ok(vec![self.make_error_box(
                    "inline images not supported!".to_owned(),
                    true,
                )])
            }
        } else {
            Ok(vec![self.make_error_box(
                "this iref type is not supported, yet!".to_owned(),
                true,
            )])
        }
    }

    pub fn export(&mut self, node: &Element) -> EdtrResult<Vec<EdtrPlugin>> {
        Ok(match node {
            Element::Document(ref doc) => vec![self.export_doc_root(doc)?.into()],
            Element::Heading(ref heading) => self.export_heading(heading)?,
            Element::Text(Text { text, .. }) => vec![text_plugin_from(text.clone())],
            Element::Formatted(formatted) => self.export_formatted_text(formatted)?,
            Element::Paragraph(par) => self.export_paragraph(par)?,
            Element::Template(template) => self.export_template(template)?,
            Element::List(list) => vec![self.export_list(list)?],
            Element::ListItem(item) => vec![self.export_list_item(item)?],
            Element::HtmlTag(tag) => self.export_htmltag(tag)?,
            Element::InternalReference(iref) => self.export_internalref(iref)?,
            _ => vec![self.make_error_box("unimplemented element!".to_owned(), false)],
        })
    }
}
