use anyhow::{Result, anyhow};
use clap::{Parser, Subcommand};
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

use crate::rendered::{Renderer, Schema as RenderSchema};
use crate::schemas::{BlockData, SchemaGenerator};
use crate::templater::Templater;
use crate::word::WordParser;

mod rendered;
mod schemas;
mod templater;
mod word;

/// autoWord: Render DOCX from Jinja-like XML templates or create templates from DOCX
#[derive(Parser, Debug)]
#[command(
    name = "autoWord",
    version,
    about = "Generate DOCX using templates, or extract templates from a DOCX"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Extract template blocks from a DOCX and store as XML (+ per-block schema JSON)
    Create {
        /// Path to source DOCX file containing block markers
        #[arg(long, value_name = "DOCX", required = true)]
        docx: PathBuf,
        /// Directory to store extracted templates (will be created if missing)
        #[arg(long, value_name = "DIR", default_value = "templates")]
        out: PathBuf,
    },
    /// Render a DOCX using templates and a render schema JSON (list of blocks + data)
    Render {
        /// Directory containing template XML files
        #[arg(long, value_name = "DIR", required = true)]
        templates: PathBuf,
        /// Path to input DOCX file to be rendered
        #[arg(long, value_name = "DOCX", required = true)]
        docx: PathBuf,
        /// Path to render schema JSON (Vec<{ block_name, block_data }>)
        #[arg(long, value_name = "JSON", required = true)]
        schema: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Create { docx, out } => {
            fs::create_dir_all(&out)?;
            let document_xml = WordParser::get_document_xml(
                docx.to_str().ok_or_else(|| anyhow!("Invalid DOCX path"))?,
            )?;

            let blocks = Templater::parse_document_xml(&document_xml)?;
            Templater::store_template(blocks.clone(), out.to_str().unwrap())?;

            let mut f = File::create(PathBuf::from(&out).join("schemas.json"))?;
            let block_data_vec: Vec<BlockData> = blocks
                .into_iter()
                .map(|(name, content)| BlockData {
                    block_name: name,
                    block_content: content,
                })
                .collect();
            let schema_json = SchemaGenerator::generate_shema(&block_data_vec)?;
            f.write_all(schema_json.as_bytes())?;

            println!("Templates and schemas created in {}", out.display());
        }
        Commands::Render {
            templates,
            docx,
            schema,
        } => {
            let schema_json = fs::read_to_string(&schema)?;
            let schemas: Vec<RenderSchema> = serde_json::from_str(&schema_json)?;

            Renderer::render(
                schemas,
                templates
                    .to_str()
                    .ok_or_else(|| anyhow!("Invalid templates path"))?,
                docx.to_str().ok_or_else(|| anyhow!("Invalid DOCX path"))?,
            )?;

            println!(
                "Successfully rendered {} -> rendered_{}",
                docx.file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("input.docx"),
                docx.file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("input.docx")
            );
        }
    }

    Ok(())
}
