use mediawiki_parser::transformations::*;
use mediawiki_parser::*;
use util::{extract_plain_text, find_arg};

/// Convert list templates (MFNF) to mediawiki lists.
pub fn convert_template_list(mut root: Element, _settings: ()) -> TResult {
    if let Element::Template(ref mut template) = root {
        let template_name = extract_plain_text(&template.name).trim().to_lowercase();
        if ["list", "liste"].contains(&template_name.as_str()) {

            let mut list_content = vec![];

            let list_type = if let Some(&Element::TemplateArgument(ref arg))
                = find_arg(&template.content, &["type".into()])
            {
                extract_plain_text(&arg.value).to_lowercase()
            } else {
                String::new()
            };

            let item_kind = match list_type.trim() {
                "ol" | "ordered" => ListItemKind::Ordered,
                "ul" | _ => ListItemKind::Unordered,
            };

            for child in template.content.drain(..) {
                if let Element::TemplateArgument(mut arg) = child {
                    if arg.name.starts_with("item") {
                        let li = Element::ListItem(ListItem {
                            position: arg.position,
                            content: arg.value,
                            kind: item_kind,
                            depth: 1,
                        });
                        list_content.push(li);

                    // a whole sublist only wrapped by the template,
                    // -> replace template by wrapped list
                    } else if arg.name.starts_with("list") {
                        if arg.value.is_empty() {
                            continue
                        }
                        let sublist = arg.value.remove(0);
                        return recurse_inplace(&convert_template_list, sublist, ());
                    }
                }
            }

            let list = Element::List(List {
                position: template.position.to_owned(),
                content: list_content,
            });
            return recurse_inplace(&convert_template_list, list, ());
        }
    }
    return recurse_inplace(&convert_template_list, root, ())
}
