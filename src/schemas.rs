use anyhow::Result;
use minijinja::machinery::{ast, parse};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Clone, Serialize)]
#[serde(tag = "kind")]
enum AstType {
    String,
    Object {
        properties: HashMap<String, AstType>,
    },
    Array {
        element_type: Box<AstType>,
    },
    Unknown,
}

impl AstType {
    fn ensure_path<'a>(&'a mut self, path: &[String]) -> &'a mut AstType {
        let mut current = self;

        for key in path {
            if let AstType::Array { element_type } = current {
                current = element_type;
            }

            if !matches!(current, AstType::Object { properties: _ }) {
                *current = AstType::Object {
                    properties: HashMap::new(),
                };
            }

            if let AstType::Object { properties } = current {
                current = properties.entry(key.clone()).or_insert(AstType::String);
            }
        }
        current
    }
}

#[derive(Clone, Serialize)]
struct Schema {
    pub block_name: String,
    pub block_data_type: AstType,
}

pub struct BlockData {
    pub block_name: String,
    pub block_content: String,
}

pub struct SchemaGenerator {}

impl SchemaGenerator {
    fn parse(source: &str, file_name: &str) -> Result<AstType> {
        let ast = parse(source, file_name, Default::default(), Default::default())?;
        let mut root_type = AstType::Unknown;
        let mut aliases = HashMap::new();

        visit_stmt(&ast, &mut root_type, &mut aliases);

        Ok(root_type)
    }

    pub fn generate_shema(data: &Vec<BlockData>) -> Result<String> {
        let mut schemas_vec: Vec<Schema> = Vec::new();
        for block in data {
            schemas_vec.push(Schema {
                block_name: block.block_name.clone(),
                block_data_type: Self::parse(&block.block_content, &block.block_name)?,
            });
        }

        Ok(serde_json::to_string_pretty(&schemas_vec)?)
    }
}

fn visit_stmt(
    stmt: &ast::Stmt,
    root_type: &mut AstType,
    aliases: &mut HashMap<String, Vec<String>>,
) {
    match stmt {
        ast::Stmt::Template(t) => {
            t.children
                .iter()
                .for_each(|c| visit_stmt(c, root_type, aliases));
        }
        ast::Stmt::EmitExpr(e) => {
            if let Some((root, subpath)) = flatten_path(&e.expr) {
                let mut real_path = resolve_alias(&root, aliases);
                real_path.extend(subpath);
                root_type.ensure_path(&real_path);
            }
        }
        ast::Stmt::ForLoop(f) => {
            if let Some((root_var, subpath)) = flatten_path(&f.iter) {
                let mut full_path = resolve_alias(&root_var, aliases);
                full_path.extend(subpath);

                let target_node = root_type.ensure_path(&full_path);
                let inner_clone = target_node.clone();
                *target_node = AstType::Array {
                    element_type: Box::new(inner_clone),
                };

                let mut loop_aliases = aliases.clone();
                if let ast::Expr::Var(spanned) = &f.target {
                    loop_aliases.insert(spanned.id.to_string(), full_path);
                    for child in &f.body {
                        visit_stmt(child, root_type, &mut loop_aliases);
                    }
                }
            }
        }
        _ => {}
    }
}

fn flatten_path(expr: &ast::Expr) -> Option<(String, Vec<String>)> {
    let mut path = Vec::new();
    let mut current = expr;

    while let ast::Expr::GetAttr(attr) = current {
        path.push(attr.name.to_string());
        current = &attr.expr;
    }

    if let ast::Expr::Var(v) = current {
        path.reverse();
        return Some((v.id.to_string(), path));
    }

    None
}

fn resolve_alias(var: &str, aliases: &HashMap<String, Vec<String>>) -> Vec<String> {
    aliases
        .get(var)
        .cloned()
        .unwrap_or_else(|| vec![var.to_string()])
}
