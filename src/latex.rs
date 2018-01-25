//! Implementation of the `latex` target.
//!
//! This target renders the final syntax tree to a LaTeX document body.
//! LaTeX boilerplate like preamble or document tags have to be added afterwards.

use std::io;
use std::io::Write;
use std::str;
use settings::Settings;
use mediawiki_parser::ast::*;
use mediawiki_parser::transformations::*;
use util::*;
use std::path;
use std::collections::HashMap;
use std::ffi::OsStr;
use target::Target;
use traversion::Traversion;


/// Data for LaTeX export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatexTarget {
    /// Does this target operate on the input tree directly or with
    /// mfnf transformations applied?
    with_transformation: bool,
    /// extension of the resulting file. Used for make dependency generation.
    target_extension: String,
    /// are dependencies generated for this target?
    generate_deps: bool,
    /// mapping of external file extensions to target extensions.
    /// this is useful if external dependencies should be processed by
    /// make for this target.
    deps_extension_mapping: HashMap<String, String>,

    /// Page trim in mm.
    page_trim: f32,
    /// Paper width in mm.
    page_width: f32,
    /// Paper height in mm.
    page_height: f32,
    /// Font size in pt.
    font_size: f32,
    /// Baseline height in pt.
    baseline_height: f32,
    /// Paper border in mm as [top, bottom, outer, inner]
    border: [f32; 4],
    /// Document class options.
    document_options: String,
    /// Indentation depth for template content.
    indentation_depth: usize,
    /// Maximum line width (without indentation).
    max_line_width: usize,
    /// Maximum width of an image in a figure as fraction of \\textwidth
    image_width: f32,
    /// Maximum height of an imgae in a figure as fraction of \\textheight
    image_height: f32,

    /// Templates which can be exported as an environment.
    /// The template may have a `title` attribute and a content
    /// attribute, which has the same name as the environment.
    /// Any additional template attributes will be exported as
    /// subsequent environments, if listed here.
    environments: HashMap<String, Vec<String>>,
}

impl Default for LatexTarget {
    fn default() -> LatexTarget {
        LatexTarget {
            with_transformation: true,
            target_extension: "tex".into(),
            generate_deps: true,
            deps_extension_mapping: string_map![
                "png" => "pdf",
                "svg" => "pdf",
                "eps" => "pdf",
                "jpg" => "pdf",
                "jpeg" => "pdf",
                "gif" => "pdf"
            ],
            page_trim: 0.0,
            page_width: 155.0,
            page_height: 235.0,
            font_size: 9.0,
            baseline_height: 12.0,
            border: [20.5, 32.6, 22.0, 18.5],
            document_options: "tocflat, listof=chapterentry".into(),
            indentation_depth: 4,
            max_line_width: 80,
            image_width: 0.5,
            image_height: 0.2,
            environments: string_value_map![
                "definition" => string_vec!["definition"],
                "theorem" => string_vec!["theorem", "explanation", "example",
                                         "proofsummary", "solutionprocess", "solution",
                                         "proof"],
                "solution" => string_vec!["solution"],
                "solutionprocess" => string_vec!["solutionprocess"],
                "proof" => string_vec!["proof"],
                "proofsummary" => string_vec!["proofsummary"],
                "alternativeproof" => string_vec!["alternativeproof"],
                "hint" => string_vec!["hint"],
                "warning" => string_vec!["warning"],
                "example" => string_vec!["example"],
                "importantparagraph" => string_vec!["importantparagraph"],
                "exercise" => string_vec!["exercise", "explanation", "example",
                                          "proofsummary", "solutionprocess", "solution",
                                          "proof"],
                "explanation" => string_vec!["explanation"]
            ],
        }
    }
}

impl Target for LatexTarget {

    fn get_name(&self) -> &str { "latex" }
    fn do_include_sections(&self) -> bool { true }
    fn do_generate_dependencies(&self) -> bool { true }
    fn get_target_extension(&self) -> &str { "tex" }
    fn get_extension_mapping(&self) -> &HashMap<String, String> {
        &self.deps_extension_mapping
    }
    fn export<'a>(&self,
                  root: &'a Element,
                  settings: &Settings,
                  out: &mut io::Write) -> io::Result<()> {

        // apply latex-specific transformations
        let mut latex_tree = root.clone();
        latex_tree = normalize_formula(latex_tree, settings)
            .expect("Could not appy LaTeX-Secific transformations!");

        let mut renderer = LatexRenderer {
            path: vec![],
            latex: &self,
        };
        renderer.run(&latex_tree, settings, out)
    }
}

/// Recursively renders a syntax tree to latex.
struct LatexRenderer<'e, 't> {
    pub path: Vec<&'e Element>,
    pub latex: &'t LatexTarget,
}

impl<'e, 's: 'e, 't: 'e> Traversion<'e, &'s Settings> for LatexRenderer<'e, 't> {
    fn path_push(&mut self, root: &'e Element) {
        self.path.push(&root);
    }
    fn path_pop(&mut self) -> Option<&'e Element> {
        self.path.pop()
    }
    fn get_path(&self) -> &Vec<&'e Element> {
        &self.path
    }
    fn work(&mut self, root: &'e Element, settings: &'s Settings,
            out: &mut io::Write) -> io::Result<bool> {

        Ok(match root {
            // Node elements
            &Element::Document { .. } => true,
            &Element::Heading { .. } => self.heading(root, settings, out)?,
            &Element::Formatted { .. } => self.formatted(root, settings, out)?,
            &Element::Paragraph { .. } => self.paragraph(root, settings, out)?,
            &Element::Template { .. } => self.template(root, settings, out)?,
            &Element::InternalReference { .. } => self.internal_ref(root, settings, out)?,
            &Element::List { .. } => self.list(root, settings, out)?,

            // Leaf Elements
            &Element::Text { .. } => self.text(root, settings, out)?,
            &Element::Comment { .. } => self.comment(root, settings, out)?,
            _ => {
                self.write_error(&format!("export for element `{}` not implemented!",
                    root.get_variant_name()), settings, out)?;
                false
            },
        })
    }
}

impl<'e, 's: 'e, 't: 'e> LatexRenderer<'e, 't> {
    fn write_error(&self,
                   message: &str,
                   settings: &Settings,
                   out: &mut io::Write) -> io::Result<()> {

        let indent = self.latex.indentation_depth;
        let line_width = self.latex.max_line_width;

        let message = escape_latex(message);
        writeln!(out, "\\begin{{error}}")?;
        writeln!(out, "{}", indent_and_trim(&message, indent, line_width))?;
        writeln!(out, "\\end{{error}}")
    }

    fn write_def_location(&self, pos: &Span, doctitle: &str,
                          out: &mut io::Write) -> io::Result<()> {

        writeln!(out, "\n% defined in {} at {}:{} to {}:{}", doctitle,
                 pos.start.line, pos.start.col,
                 pos.end.line, pos.end.col)
    }

    fn template(&mut self, root: &'e Element,
                       settings: &'s Settings,
                       out: &mut io::Write) -> io::Result<bool> {

        if let &Element::Template { ref name, ref content, ref position } = root {

            let template_name;
            if let Some(&Element::Text { ref text, .. }) = name.first() {
                template_name = text;
            } else {
                self.write_error("Template names must be text-only!", settings, out)?;
                return Ok(false);
            };

            let doctitle = &settings.document_title;
            let envs = &self.latex.environments;

            // export simple environment templates
            if let Some(envs) = envs.get(template_name) {
                let title_content = find_arg(content, "title");

                self.write_def_location(position, doctitle, out);

                for environment in envs {
                    if let Some(env_content) = find_arg(content, environment) {
                        write!(out, "\\begin{{{}}}[", environment)?;
                        if let Some(title_content) = title_content {
                            self.run(title_content, settings, out)?;
                        }
                        write!(out, "]\n")?;

                        self.run(env_content, settings, out)?;
                        write!(out, "\\end{{{}}}\n", environment)?;
                    }
                }
                return Ok(false);
            }

            // any other template
            match &template_name[..] {
                "formula" => {
                    let mut math_text = "ERROR: Template was not transformed properly!";
                    if let Some(&Element::TemplateArgument { ref value, .. }) = content.first() {
                        if let Some(&Element::Text {ref text, .. }) = value.first() {
                            math_text = trim_enclosing(text.trim(),
                                                    "\\begin{align}",
                                                    "\\end{align}");
                            math_text = trim_enclosing(math_text,
                                                    "\\begin{align*}",
                                                    "\\end{align*}").trim();
                        };
                    };
                    let indent = self.latex.indentation_depth;
                    let width= self.latex.max_line_width;

                    writeln!(out, "{}", "\\begin{align*}")?;
                    writeln!(out, "{}", indent_and_trim(math_text, indent, width))?;
                    writeln!(out, "{}", "\\end{align*}")?;
                },
                "anchor" => {
                    write!(out, " {} ", escape_latex("<no anchors yet!>"))?;
                }
                _ => {
                    let message = format!("MISSING TEMPLATE: {}\n{} at {}:{} to {}:{}",
                                        template_name, &doctitle,
                                        position.start.line, position.start.col,
                                        position.end.line, position.end.col);
                    self.write_error(&message, settings, out)?;
                }
            };
        }
        Ok(false)
    }

    fn internal_ref(&mut self, root: &'e Element,
                    settings: &'s Settings,
                    out: &mut io::Write) -> io::Result<bool> {

        if let &Element::InternalReference { ref target, ref options,
                                            ref caption, ref position } = root {

            let target_str = extract_plain_text(target);
            let target_path = path::Path::new(&target_str);
            let ext = target_path.extension().unwrap_or(OsStr::new(""));
            let ext_str = ext.to_os_string().into_string().unwrap_or(String::new());

            let doctitle = &settings.document_title;
            let img_exts = &settings.image_extensions;


            // file is an image
            if img_exts.contains(&ext_str) {

                let width = self.latex.image_width;
                let height = self.latex.image_height;
                let indent = self.latex.indentation_depth;
                let line_width = self.latex.max_line_width;
                let image_path = &settings.image_path;

                let image_path = path::Path::new(image_path)
                    .join(target_path.file_stem().expect("image path is empty!"))
                    .to_string_lossy()
                    .to_string();
                let image_path = filename_to_make(&image_path);

                // collect image options
                let mut image_options = vec![];
                for option in options {
                    image_options.push(extract_plain_text(&option).trim().to_string());
                }

                self.write_def_location(position, &doctitle, out);

                writeln!(out, "\\begin{{figure}}[h]")?;

                // render caption content
                let mut cap_content = vec![];
                writeln!(&mut cap_content, "% image options: {:?}", &image_options)?;
                writeln!(&mut cap_content, "\\adjincludegraphics[max width={}\\textwidth, \
                                            max height={}\\textheight]{{{}}}",
                    width, height, &image_path)?;

                write!(&mut cap_content, "\\caption{{")?;
                self.run_vec(caption, settings, &mut cap_content)?;
                write!(&mut cap_content, "}}")?;

                writeln!(out, "{}",
                    &indent_and_trim(&str::from_utf8(&cap_content).unwrap(),
                        indent, line_width))?;
                writeln!(out, "\\end{{figure}}\n")?;

                return Ok(false)
            }
            let msg = format!("No export function defined for target {:?}", target_path);
            self.write_error(&msg, settings, out)?;
        }
        Ok(false)
    }


    fn paragraph(&mut self, root: &'e Element,
                 settings: &'s Settings,
                 out: &mut io::Write) -> io::Result<bool> {

        if let &Element::Paragraph { ref content, .. } = root {

            // render paragraph content
            let mut par_content = vec![];
            self.run_vec(content, settings, &mut par_content)?;
            let par_string = str::from_utf8(&par_content)
                .unwrap().trim_right().to_string();

            let indent = self.latex.indentation_depth;
            let line_width = self.latex.max_line_width;

            // trim and indent output string
            let trimmed = indent_and_trim(&par_string, indent, line_width);
            writeln!(out, "{}\n", &trimmed)?;
        };
        Ok(false)
    }

    fn heading(&mut self, root: &'e Element,
               settings: &'s Settings,
               out: &mut io::Write) -> io::Result<bool> {

        if let &Element::Heading {ref depth, ref caption, ref content, .. } = root {

            write!(out, "\\")?;

            for _ in 1..*depth {
                write!(out, "sub")?;
            }

            write!(out, "section{{")?;
            self.run_vec(caption, settings, out)?;
            write!(out, "}}\n\n")?;

            self.run_vec(content, settings, out)?;
        };
        Ok(false)
    }

    fn list(&mut self, root: &'e Element,
            settings: &'s Settings,
            out: &mut io::Write) -> io::Result<bool> {

        if let &Element::List { ref content, .. } = root {

            let kind = if let &Element::ListItem { ref kind, .. } =
                content.first().unwrap_or(root) {
                    kind
            } else {
                self.write_error("first child of list element \
                    is not a list item!", settings, out)?;
                return Ok(false)
            };

            let envname = match kind {
                &ListItemKind::Ordered => "enumerate",
                &ListItemKind::Unordered => "itemize",
                &ListItemKind::Definition => "itemize",
                &ListItemKind::DefinitionTerm => "itemize",
            };
            writeln!(out, "\\begin{{{}}}", envname)?;

            let mut def_term_temp = String::new();

            for child in content {
                if let &Element::ListItem { ref content, ref kind, .. } = child {

                    // render paragraph content
                    let mut par_content = vec![];
                    self.run_vec(content, settings, &mut par_content)?;
                    let par_string = str::from_utf8(&par_content)
                        .unwrap().trim_right().to_string();

                    // definition term
                    if let &ListItemKind::DefinitionTerm = kind {
                        def_term_temp.push_str(&par_string);
                        continue
                    }

                    let item_string = if let &ListItemKind::Definition = kind {
                        format!("\\item \\textbf{{{}}}: {}", def_term_temp, par_string)
                    } else {
                        format!("\\item {}", par_string)
                    };
                    def_term_temp = String::new();


                    let indent = self.latex.indentation_depth;
                    let line_width = self.latex.max_line_width;

                    // trim and indent output string
                    let trimmed = indent_and_trim(&item_string, indent, line_width);

                    writeln!(out, "{}", &trimmed)?;
                }
            }
            writeln!(out, "\\end{{{}}}\n", envname)?;
        };
        Ok(false)
    }

    fn formatted(&mut self, root: &'e Element,
                 settings: &'s Settings,
                 out: &mut io::Write) -> io::Result<bool> {

        if let &Element::Formatted { ref markup, ref content, .. } = root {
            match markup {
                &MarkupType::NoWiki => {
                    self.run_vec(content, settings, out)?;
                },
                &MarkupType::Bold => {
                    write!(out, "\\textbf{{")?;
                    self.run_vec(content, settings, out)?;
                    write!(out, "}}")?;
                },
                &MarkupType::Italic => {
                    write!(out, "\\textit{{")?;
                    self.run_vec(content, settings, out)?;
                    write!(out, "}}")?;

                },
                &MarkupType::Math => {
                    write!(out, "${}$", match content.first() {
                        Some(&Element::Text {ref text, .. }) => text,
                        _ => "parse error!",
                    })?;
                },
                _ => {
                    let msg = format!("MarkupType not implemented: {:?}", &markup);
                    self.write_error(&msg, settings, out)?;
                }
            }
        }
        Ok(false)
    }

    fn comment(&mut self, root: &'e Element,
               settings: &'s Settings,
               out: &mut io::Write) -> io::Result<bool> {

        if let &Element::Comment { ref text, .. } = root {
            writeln!(out, "% {}", text)?;
        }
        Ok(false)
    }

    fn text(&mut self, root: &'e Element,
            settings: &'s Settings,
            out: &mut io::Write) -> io::Result<bool> {

        if let &Element::Text { ref text, .. } = root {
            write!(out, "{}", &escape_latex(text))?;
        }
        Ok(false)
    }
}


/// Transform a formula template argument to text-only.
pub fn normalize_formula(mut root: Element, settings: &Settings) -> TResult {
    if let &mut Element::Template { ref name, ref mut content, ref position, .. } = &mut root {
        if let Some(&Element::Text {ref text, .. }) = name.first() {
            if text == "formula" {
                let arg_error = Element::Error {
                    position: position.clone(),
                    message: "Forumla templates must have exactly one anonymous argument, \
                                which is LaTeX source code entirely enclosed in <math></math>!".to_string(),
                };

                if content.len() != 1 {
                    return Ok(arg_error);
                }
                if let Some(&mut Element::TemplateArgument {ref mut value, .. }) = content.first_mut() {
                    if value.len() != 1 {
                        return Ok(arg_error);
                    }
                    if let Some(Element::Formatted { ref markup, ref mut content, .. }) = value.pop() {
                        if content.len() != 1 || if let &MarkupType::Math = markup {false} else {true} {
                            return Ok(arg_error);
                        }
                        value.clear();
                        value.append(content);
                    } else {
                        return Ok(arg_error);
                    }
                } else {
                    return Ok(arg_error);
                }
            }
        }
    };
    recurse_inplace(&normalize_formula, root, settings)
}


