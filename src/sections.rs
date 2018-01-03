use std::io;
use std::str;
use settings::Settings;
use mediawiki_parser::ast::*;
use util::*;

/// Write all section names encountered to an output file.
pub fn collect_section_names<'a>(root: &'a Element,
                                 path: &mut Vec<&'a Element>,
                                 settings: &Settings,
                                 out: &mut io::Write) -> io::Result<()> {

    if let &Element::HtmlTag { ref name, ref attributes, .. } = root {
        if name.to_lowercase() == "section"  {
            for attr in attributes {
                if attr.key == "begin" {
                    writeln!(out, "{}", attr.value)?;
                }
            }
        }
    };
    traverse_with(&collect_section_names, root, path, settings, out)?;
    Ok(())
}

pub fn collect_sections<'a>(root: &'a Element,
                            _path: &mut Vec<&'a Element>,
                            settings: &Settings,
                            out: &mut io::Write) -> io::Result<()> {

    let mut sections_string = vec![];
    collect_section_names(root, &mut vec![], settings, &mut sections_string)?;
    let sections_string = String::from_utf8(sections_string).unwrap();

    // list of section names defined in the document
    let sections: Vec<&str> = sections_string.split("\n").collect();

    eprintln!("sections: {:?}", &sections);

    for section in sections {
        if section.is_empty() {
            continue
        }
        let mut find_settings = SectionData {
            begin: true,
            label: section,
        };

        let mut start = vec![];
        let mut end = vec![];
        if let Ok(()) = find_section(root, &mut start, &find_settings, &mut vec![]) {
            continue
        }

        find_settings.begin = false;
        if let Ok(()) = find_section(root, &mut end, &find_settings, &mut vec![]) {
            continue
        }

        if !start.is_empty() && !end.is_empty() {
            eprintln!("get_intermediary for {}", section);
            let inter = get_intermediary(&start, &end);
        } else {

        }
    }
    Ok(())
}

fn get_intermediary<'a>(start: &Vec<&'a Element>, end: &Vec<&'a Element>) -> Vec<&'a Element> {
    // lowest common node
    let mut common = None;
    for ps in start.iter().rev() {
        for pe in end.iter().rev() {
            if pe == ps {
                common = Some(ps);
                break;
            }
        }
        if let Some(_) = common {
            break;
        }
    }
    if let None = common {
        return vec![];
    }
    let common = common.unwrap();
    eprintln!("lowest common: {}", common.get_variant_name());
    vec![]
}

/// Parameters for session finding.
struct SectionData<'a> {
    pub label: &'a str,
    pub begin: bool,
}

/// Return a path to the start / end of a section
fn find_section<'a>(root: &'a Element,
                    path: &mut Vec<&'a Element>,
                    settings: &SectionData<'a>,
                    out: &mut io::Write) -> io::Result<()> {

    path.push(root);
    if let &Element::HtmlTag { ref name, ref attributes, .. } = root {
        if name.to_lowercase() == "section" {
            for attr in attributes {
                if attr.key.to_lowercase() == if settings.begin {"begin"} else {"end"}
                    && attr.value.to_lowercase() == settings.label.to_lowercase() {
                    // abort recursion, preserve path
                    return Err(io::Error::new(io::ErrorKind::Other, "recusion abort"));
                }
            }
        }
    };
    // pop if recursion not aborted
    traverse_with(&find_section, root, path, settings, out)?;
    path.pop();
    Ok(())
}
