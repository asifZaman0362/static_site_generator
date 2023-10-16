#[allow(deprecated)]
use chrono::{Date, Utc};
use comrak::nodes::{Ast, AstNode};
use comrak::{self, format_html, parse_document, Arena, Options};
use regex::{Captures, Regex};
use std::cell::RefCell;
use std::env::args;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::process::Command;

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

#[allow(deprecated)]
fn main() -> std::io::Result<()> {
    let args: Vec<_> = args().collect();
    if args.len() != 3 {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "not enough arguments or too many!",
        ))
    } else {
        let time: Date<Utc> = Utc::today();
        let iso_date = time.format("%Y-%m-%d").to_string();
        let date_display_format = time.format("%b %d, %Y").to_string();

        let filename = args.get(1).unwrap();
        let path = Path::new(filename);
        let mut file = File::open(path)?;
        let mut buf = vec![];
        file.read_to_end(&mut buf)?;

        // Parse document
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

        let arena = Arena::new();
        let root = parse_document(&arena, &String::from_utf8(buf).unwrap(), &options);
        let regex = Regex::new(r"#(\w+)").unwrap();
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

        format_html(root, &options, &mut html).unwrap();
        let html_string = String::from_utf8(html).unwrap();
        let replaced = regex.replace_all(html_string.as_str(), |caps: &Captures| {
            let tag_name = &caps[1];
            format!("<tag>{tag_name}</tag>")
        });

        // put article inside skeleton
        let outdir = args.get(2).unwrap();
        let mut skeleton = File::open(format!("{outdir}/skeleton.html"))?;
        let mut buf = vec![];
        skeleton.read_to_end(&mut buf)?;
        let string = String::from_utf8(buf).unwrap();
        let (before, after) = string.split_once("<article></article>").unwrap();
        let article = replaced.to_string();
        let joined = format!("{before}<article id=\"post\">{article}</article>{after}");

        // write file out
        let date_for_filename = time.format("%Y-%b").to_string();
        let (year, month) = date_for_filename.split_once("-").unwrap();
        let postname = path.file_stem().unwrap().to_str().unwrap();
        let outdir_tagged = format!("{outdir}/{year}/{month}");
        std::fs::create_dir_all(&outdir_tagged)?;
        let outfilepath = format!("{outdir_tagged}/{postname}.html");
        let mut outfile = File::create(&outfilepath)?;
        outfile.write_all(joined.as_bytes())?;

        print!("{}", outfilepath);

        let prettier = "prettier";
        let args = [outfilepath, "-w".to_string()];

        Command::new(prettier).args(args).output().unwrap();

        Ok(())
    }
}
