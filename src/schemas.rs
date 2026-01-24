use anyhow::Result;
use minijinja::machinery::{ast, parse};
use serde::Serialize;
use std::{collections::HashMap, fs::File, io::Write, path::Path};

#[derive(Clone, Serialize)]
enum Type {
    Var,
    Object(HashMap<String, Type>),
    Array(Box<Type>),
    Unknown,
}

impl Type {
    fn ensure_path<'a>(&'a mut self, path: &[String]) -> &'a mut Type {
        let mut current = self;

        for key in path {
            if let Type::Array(inner) = current {
                current = inner;
            }

            if !matches!(current, Type::Object(_)) {
                *current = Type::Object(HashMap::new());
            }

            if let Type::Object(map) = current {
                current = map.entry(key.clone()).or_insert(Type::Var);
            }
        }
        current
    }
}

#[derive(Serialize)]
struct Schema {
    block_name: String,
    block_type: Type,
}

pub struct SchemaGenerator {}

impl SchemaGenerator {
    pub fn parse(source: &str, file_name: &str) -> Result<String> {
        let ast = parse(source, file_name, Default::default(), Default::default())?;
        let mut root_schema = Type::Unknown;
        let mut aliases = HashMap::new();

        visit_stmt(&ast, &mut root_schema, &mut aliases);

        Ok(serde_json::to_string_pretty(&root_schema)?)
    }
    pub fn store_schema(block_name: &str, block_type: Type, templates_path: &str) -> Result<()> {
        let mut file = File::create(Path::new(templates_path).join(format!("{block_name}.json")))?;

        let schema = Schema {
            block_name: block_name.to_string(),
            block_type: block_type
        };
        file.write_all(serde_json::to_vec_pretty(&schema)?.as_slice())?;
        Ok(())
    }
}

fn visit_stmt<'a>(
    stmt: &'a ast::Stmt,
    root_schema: &mut Type,
    aliases: &mut HashMap<String, Vec<String>>,
) {
    match stmt {
        ast::Stmt::Template(t) => {
            t.children
                .iter()
                .for_each(|c| visit_stmt(c, root_schema, aliases));
        }
        ast::Stmt::EmitExpr(e) => {
            if let Some((root, subpath)) = flatten_path(&e.expr) {
                let mut real_path = resolve_alias(&root, aliases);
                real_path.extend(subpath);
                root_schema.ensure_path(&real_path);
            }
        }
        ast::Stmt::ForLoop(f) => {
            if let Some((root_var, subpath)) = flatten_path(&f.iter) {
                let mut full_path = resolve_alias(&root_var, aliases);
                full_path.extend(subpath);

                let target_node = root_schema.ensure_path(&full_path);
                let inner_clone = target_node.clone();
                *target_node = Type::Array(Box::new(inner_clone));

                let mut loop_aliases = aliases.clone();
                if let ast::Expr::Var(spanned) = &f.target {
                    loop_aliases.insert(spanned.id.to_string(), full_path);
                    for child in &f.body {
                        visit_stmt(child, root_schema, &mut loop_aliases);
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
