use std::path::Path;
use std::collections::HashMap;
use roxmltree::{Document, Node};
use std::fs::File;
use std::io::Write;
use anyhow::{Result, anyhow};

pub struct Templater {}

enum BlockParserState {
    Searching,
    Block { name: String, content: Vec<String> },
}

impl Templater {
    pub fn parse_document_xml(xml_content: &str) -> Result<HashMap<String, String>> {
        let doc = Document::parse(xml_content)?;
        let mut blocks: HashMap<String, String> = HashMap::new();

        let mut state = BlockParserState::Searching;

        let body = doc
            .descendants()
            .find(|n| n.has_tag_name("body"))
            .ok_or(anyhow!("Body not found"))?;

        for node in body.children() {
            if !node.is_element() {
                continue;
            }
            let text_content = Self::get_node_text(node);
            let raw_xml = &xml_content[node.range()];

            match state {
                BlockParserState::Searching => {
                    if text_content.starts_with("#! BLOCK:") {
                        let name = text_content.replace("#! BLOCK:", "").trim().to_string();
                        state = BlockParserState::Block {
                            name,
                            content: Vec::new(),
                        };
                    }
                }
                BlockParserState::Block {
                    ref name,
                    ref mut content,
                } => {
                    if text_content.trim_start().starts_with("#! ENDBLOCK") {
                        blocks.insert(name.clone(), content.join("\n"));
                        state = BlockParserState::Searching;
                    } else {
                        content.push(Self::escape_xml_tag(text_content.as_str(), raw_xml));
                    }
                }
            }
        }

        Ok(blocks)
    }

    pub fn store_template(blocks: HashMap<String, String>, template_path: &str) -> Result<()> {
        for (name, content) in blocks {
            let mut file = File::create(Path::new(template_path).join(format!("{name}.xml")))?;
            file.write_all(content.as_bytes())?;
        }
        Ok(())
    }

    fn get_node_text(node: Node) -> String {
        node.descendants()
            .filter(|n| n.is_text())
            .map(|n| n.text().unwrap_or(""))
            .collect()
    }

    fn escape_xml_tag(text: &str, raw_xml: &str) -> String {
        let trimmed_text = text.trim_start();
        if trimmed_text.starts_with("#!") {
            return text
                .replacen("#!", "", 1)
                .replace("“", "\"")
                .replace("”", "\"");
        }
        raw_xml.to_string()
    }
}
