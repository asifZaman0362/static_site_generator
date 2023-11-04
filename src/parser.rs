use chrono::prelude::*;
use std::cell::RefCell;
use comrak::nodes::{Ast, AstNode};
use comrak::{self, format_html_with_plugins, parse_document, Arena, Options};
use regex::{Regex, Captures};

#[allow(dead_code)]
fn iter_nodes<'a, F>(node: &'a AstNode<'a>, f: &F)
where
    F: Fn(&'a AstNode<'a>),
{
    f(node);
    for c in node.children() {
        iter_nodes(c, f);
    }
}

pub fn to_html(bytes: Vec<u8>) -> String {
    let mut options = Options::default();
    options.render.unsafe_ = true;
    options.extension.tasklist = true;
    options.extension.table = true;
    options.extension.superscript = true;
    options.extension.strikethrough = true;
    options.extension.description_lists = true;
    options.extension.footnotes = true;
    options.render.github_pre_lang = true;
    options.extension.header_ids = Some("section-".to_string());

    let time = Utc::now();
    let iso_date = time.format("%Y-%m-%d").to_string();
    let date_display_format = time.format("%b %d, %Y").to_string();

    let arena = Arena::new();
    let root = parse_document(&arena, &String::from_utf8(bytes).unwrap(), &options);
    let regex = Regex::new(r"#t_(\w+)").unwrap();
    let mut html = vec![];
    let time_html = format!("<time datetime={}>{}</time>", iso_date, date_display_format);
    let parent = AstNode::new(RefCell::new(Ast::new(
        comrak::nodes::NodeValue::HtmlBlock(comrak::nodes::NodeHtmlBlock {
            block_type: 0,
            literal: time_html,
        }),
        comrak::nodes::LineColumn { line: 0, column: 0 },
    )));

    iter_nodes(root, &|node| {
        let n = node.data.borrow_mut();
        match n.value {
            comrak::nodes::NodeValue::Heading(heading) => {
                if heading.level == 1 {
                    node.insert_after(&parent);
                }
            }
            _ => {}
        }
    });

    let syntect = comrak::plugins::syntect::SyntectAdapter::new("base16-eighties.dark");
    let mut plugins = comrak::Plugins::default();
    plugins.render.codefence_syntax_highlighter = Some(&syntect);

    format_html_with_plugins(root, &options, &mut html, &plugins).unwrap();
    let html_string = String::from_utf8(html).unwrap();
    let replaced = regex.replace_all(html_string.as_str(), |caps: &Captures| {
        let tag_name = &caps[1];
        format!("<tag>{tag_name}</tag>")
    });
    replaced.to_string()
}
