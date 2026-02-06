use anyhow::{Error, Result};
use roxmltree::Document;
use serde::Deserialize;
use serde_json::Value;
use std::{fs::File, io::Read, path::PathBuf};

use crate::word;

#[derive(Deserialize, Debug)]
pub struct Schema {
    block_name: String,
    #[serde(default)]
    block_data: Value,
}

pub struct Renderer {}

impl Renderer {
    pub fn render(schemas: Vec<Schema>, template_path: &str, document_path: &str) -> Result<()> {
        let document_xml = word::WordParser::get_document_xml(document_path)?;
        let doc = Document::parse(document_xml.as_str())?;
        let body = doc
            .descendants()
            .find(|n| n.has_tag_name("body"))
            .ok_or(Error::msg("Render Error: body not found"))?;

        let sect_pr_xml = body
            .children()
            .find(|n| n.has_tag_name("sectPr"))
            .map(|n| &document_xml[n.range()])
            .unwrap_or("");

        let rendered_blocks = Self::render_blocks(&schemas, template_path)?;
        let mut body_open_tag = String::from("<w:body");
        for attr in body.attributes() {
            body_open_tag.push_str(&format!(" {}=\"{}\"", attr.name(), attr.value()));
        }
        body_open_tag.push('>');
        let new_body_content = format!(
            "{}{}{}</w:body>",
            body_open_tag,
            rendered_blocks.join(""),
            sect_pr_xml
        );

        let prefix = &document_xml[..body.range().start];
        let suffix = &document_xml[body.range().end..];

        let document_path = PathBuf::from(document_path);
        let mut rendered_document_path = document_path.clone();
        rendered_document_path.set_file_name(format!(
            "rendered_{}",
            document_path
                .file_name()
                .ok_or(Error::msg("Render error: Document without file name"))?
                .to_str()
                .ok_or(Error::msg("Render error: Failed parsing document name"))?
        ));
        word::WordParser::update_document_xml(
            document_path
                .to_str()
                .ok_or(Error::msg("Render Error: failed converting path to str"))?,
            rendered_document_path
                .to_str()
                .ok_or(Error::msg("Render Error: failed converting path to str"))?,
            format!("{}{}{}", prefix, new_body_content, suffix).as_str(),
        )?;
        Ok(())
    }

    fn render_blocks(schemas: &Vec<Schema>, template_path: &str) -> Result<Vec<String>> {
        let mut env = minijinja::Environment::new();
        for schema in schemas {
            let mut file = File::open(
                PathBuf::from(template_path).join(format!("{}.xml", schema.block_name)),
            )?;
            let mut buf = String::new();
            file.read_to_string(&mut buf)?;
            env.add_template_owned(schema.block_name.clone(), buf)?;
        }

        let mut result = Vec::new();
        for schema in schemas {
            let template = env.get_template(&schema.block_name)?;
            let rendered = template.render(&schema.block_data)?;
            result.push(rendered);
        }

        Ok(result)
    }
}
