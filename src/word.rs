use anyhow::{Result, anyhow};
use regex::Regex;
use std::fs::File;
use std::io::{Read, Write};
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

pub struct WordParser {}

impl WordParser {
    pub fn get_document_xml(path: &str) -> Result<String> {
        let file = File::open(path)?;
        let mut archive = ZipArchive::new(file)?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let name = file.name().to_string();

            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)?;

            if name == "word/document.xml" {
                let string_data = String::from_utf8(buffer)?;
                return Ok(sanitize_broken_tags(&string_data));
            }
        }

        Err(anyhow!("document.xml not found"))
    }

    pub fn update_document_xml(
        document_path: &str,
        output_path: &str,
        new_document_xml: &str,
    ) -> Result<()> {
        let file_in = File::open(document_path)?;
        let mut zip_in = ZipArchive::new(file_in)?;

        let file_out = File::create(output_path)?;
        let mut zip_out = ZipWriter::new(file_out);

        for i in 0..zip_in.len() {
            let mut file = zip_in.by_index(i)?;
            let name = file.name().to_string();

            let options = SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated)
                .unix_permissions(file.unix_mode().unwrap_or(0o755));

            if name == "word/document.xml" {
                zip_out.start_file(&name, options)?;
                zip_out.write_all(new_document_xml.as_bytes())?;
            } else {
                zip_out.start_file(&name, options)?;
                std::io::copy(&mut file, &mut zip_out)?;
            }
        }

        zip_out.finish()?;
        Ok(())
    }
}

fn sanitize_broken_tags(xml: &str) -> String {
    let re = Regex::new(r"\{(?:<[^>]+>)*?\{(.*?)\}(?:<[^>]+>)*?\}").unwrap();

    re.replace_all(xml, |caps: &regex::Captures| {
        let raw_variable = &caps[1]; // Тепер змінна в 1-й групі
        let clean_var_name = remove_xml_tags(raw_variable);
        format!("<w:t>{{{{ {} }}}}</w:t>", clean_var_name.trim())
    })
    .to_string()
}

fn remove_xml_tags(text: &str) -> String {
    let re = Regex::new(r"<[^>]*>").unwrap();
    re.replace_all(text, "").to_string()
}
