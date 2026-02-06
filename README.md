![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
# DocxInfer
DocxInfer - a cli tool written in Rust, that allows you to convert .docx file filled with Jinja2 markup into json file describing variables set in the markup. Mainly designed to allow LLMs to fill strictly styled documents.

## Key features
- Preprocessing XML with regex to fix fragmented jinja tags, followed by parsing it with roxmltree
- Generation of schemas in json format based on AST parsing.
- Loop support

## How it works
- Get document.xml file from your docx
- Parse it for blocks
- Move jinja markup outside xml tags, along with restoring their structure if Word has split them.
- Save blocks xml templates into template dir.
- Using minijinja ast parse this xmls and create block schema
- Using llm response schema and original docx render all blocks and return new .docx file

## How to use it
- Get it on your computer
- Create your template docx
![template docx example](images/template_example.png)
- run 
```sh
docx-infer create --docx <your_docx_file_path>
```
- find templates and schemas in ./templates dir
```
templates
 ┣ hello world.xml
 ┣ main.xml
 ┗ schemas.json
```
- Provide llm with schemas and necessary context
Example `schemas.json`
```json
[
  {
    "block_name": "main",
    "block_data_type": {
      "kind": "Object",
      "properties": {
        "title": {
          "kind": "String"
        }
      }
    }
  },
  {
    "block_name": "hello world",
    "block_data_type": {
      "kind": "Object",
      "properties": {
        "persons": {
          "kind": "Array",
          "element_type": {
            "kind": "String"
          }
        },
        "author": {
          "kind": "String"
        }
      }
    }
  }
]
```
(Var shall be interpreted as any string)
- receive render schema from LLM in format
``` json
[
  {
    "block_name": "main",
    "block_data": {
      "title": "Exampling Render Tool Demonstration"
    }
  },
  {
    "block_name": "hello world",
    "block_data": {
      "author": "Oleh",
      "persons": [
        "GitHub Community",
        "Rust Community"
      ]
    }
  }
]
```
- render it running
```sh
docx-infer render --templates ./templates --docx ./example_template.docx --schema ./llm_output.json
```
- view result in rendereв_<your_docx_file_name>.docx
![rendered docx example](images/rendered_example.png)

## Note
This is a personal project that solves my personal use cases, shared in hopes it might be useful for someone. For now I'm not planning new features.