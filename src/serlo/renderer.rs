use super::{SerloArgs, SerloTarget};
use serlo_he_spec::*;
use crate::preamble::*;
use uuid;

pub struct SerloRenderer<'t, 's, 'a> {
    pub target: &'t SerloTarget,

    pub settings: &'s Settings,
    pub args: &'a SerloArgs,
    pub id_count: usize,
}

impl<'s, 't, 'a> SerloRenderer<'s, 't, 'a> {
    pub fn uuid(&mut self) -> uuid::Uuid {
        self.id_count += 1;
        uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_URL, &self.id_count.to_be_bytes())
    }

    pub fn dispatch_plugin(&mut self, root: &Element) -> HEPluginInstance<Plugins> {
        match root {
            Element::Document(doc) => {
                let variant: Plugins = self.document(doc).into();
                variant.into()
            },
            _ => {
                let variant: Plugins = self.markdown_from(&format!("Not implemented: {}", root.get_variant_name())).into();
                variant.into()
            }
        }
    }

    pub fn document(&mut self, doc: &Document) -> HeHeading {
        HeHeading {
            id: self.uuid(),
            caption: self.title_from(&self.args.document_title).into(),
            content: doc.content.iter().map(|elem| self.dispatch_plugin(elem)).collect()
        }
    }

    pub fn title_from(&mut self, content: &str) -> HeTitle {
        HeTitle {
            id: self.uuid(),
            content: TitleText::from_str(content)
        }.into()
    }

    pub fn markdown_from(&mut self, content: &str) -> HeMarkdown {
        HeMarkdown {
            id: self.uuid(),
            content: MarkdownText::from_str(content)
        }.into()
    }

    pub fn run(&mut self, root: &Element) -> HEPluginInstance<Plugins> {
        self.dispatch_plugin(root)
    }
}


impl<'s, 't, 'a> SerloRenderer<'t, 's, 'a> {
    pub fn new(
        target: &'t SerloTarget,
        settings: &'s Settings,
        args: &'a SerloArgs,
    ) -> SerloRenderer<'t, 's, 'a> {
        SerloRenderer {
            id_count: 0,
            target,
            settings,
            args,
        }
    }
}
