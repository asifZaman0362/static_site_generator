#[allow(deprecated)]
use chrono::{Date, Utc};
use comrak::nodes::{Ast, AstNode};
use comrak::{self, format_html_with_plugins, parse_document, Arena, Options};
use regex::{Captures, Regex, RegexBuilder};
use std::cell::RefCell;
use std::env::args;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::process::Command;
use walkdir::WalkDir;

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

fn build_tree(path: &Path, level: usize) -> std::io::Result<String> {
    let mut heading = format!(
        "<li><details {}><summary><h{}>{}</h{}></summary>",
        if level <= 1 { "open" } else { "open" },
        level + 1,
        path.file_name().unwrap().to_str().unwrap(),
        level + 1,
    );
    let mut closing = "</details></li>".to_string();
    if level == 0 {
        heading = r#"<h1>Timeline</h1>"#.to_string();
        closing = "".to_string();
    }
    if path.is_dir() {
        let mut ul = String::from("<ul>");
        let mut i = 0;
        let mut entries = std::fs::read_dir(path)?
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, std::io::Error>>()?;
        entries.sort();
        for entry in entries.iter().rev() {
            i += 1;
            let path = entry;
            if path.is_dir() {
                ul += build_tree(&path, level + 1)?.as_str();
            } else {
                let filename = path.file_stem().unwrap().to_str().unwrap();
                let (_, path) = path.to_str().unwrap().split_once("static").unwrap();
                ul += format!(
                    "<li><h4><a href=\"{}\">{}</a></h4></li>",
                    path,
                    filename
                )
                .as_str();
            }
        }
        if i > 0 {
            heading = format!("{heading}{ul}</ul>{closing}");
        } else {
            heading = format!("{heading}{closing}");
        }
        Ok(heading)
    } else {
        let filename = path.file_stem().unwrap().to_str().unwrap();
        let (_, path) = path.to_str().unwrap().split_once("static").unwrap();
        Ok(format!(
            "<li><h4><a href=\"/blog/{}\">{}</a></h4></li>",
            path,
            filename
        ))
    }
}

fn extract_excerpt(path: &str) -> std::io::Result<String> {
    let mut buf = String::new();
    let mut file = File::open(path)?;
    file.read_to_string(&mut buf)?;
    let re = RegexBuilder::new(r#"(?s)<article id="post">.*?<h1>.*?</a.*>(.*?)</h1>.*?<p>(.*?)</p>.*?<p>(.*?)</p>.*</article>"#).build().unwrap();
    let captures = re.captures(&buf);
    if let Some(captures) = captures {
        if let (Some(h1), Some(tag), Some(p)) = (captures.get(1), captures.get(2), captures.get(3))
        {
            let (_, path) = path.split_once("static").unwrap();
            return Ok(format!(
                "<article><h1><a href=\"{}\">{}</h1></a><p>{}</p><p>{}</p></article></a>",
                path,
                h1.as_str(),
                tag.as_str(),
                p.as_str()
            ));
        }
    }
    Ok("".to_string())
}

fn get_recent(dir: &str) -> std::io::Result<String> {
    let mut string = "".to_string();
    let mut count = 0;
    for file in WalkDir::new(dir).into_iter() {
        let file = file?;
        if count >= 4 {
            break;
        }
        if file.path().is_file() {
            count += 1;
            println!("{}", file.path().to_str().unwrap());
            string += extract_excerpt(file.path().to_str().unwrap())?.as_str();
        }
    }
    Ok(string)
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

        let syntect = comrak::plugins::syntect::SyntectAdapter::new("base16-eighties.dark");
        let mut plugins = comrak::Plugins::default();
        plugins.render.codefence_syntax_highlighter = Some(&syntect);

        format_html_with_plugins(root, &options, &mut html, &plugins).unwrap();
        let html_string = String::from_utf8(html).unwrap();
        /*let replaced = regex.replace_all(html_string.as_str(), |caps: &Captures| {
            let tag_name = &caps[1];
            format!("<tag>{tag_name}</tag>")
        });*/

        // put article inside skeleton
        let outdir = args.get(2).unwrap();
        let outdir_path = Path::new(outdir);
        let posts_dir = outdir_path.join("posts");
        let tree = build_tree(&posts_dir, 0)?;
        let mut index_skeleton = File::open(format!("{outdir}/index_skeleton.html"))?;
        let mut buf = String::new();
        index_skeleton.read_to_string(&mut buf)?;
        let featured_list = get_recent(format!("{outdir}/posts").as_str())?;
        let (featured_start, featured_end) = buf
            .split_once("<section id=\"featured\"></section>")
            .unwrap();
        let (timeline_start, timeline_end) = featured_end
            .split_once("<aside id=\"timeline\"></aside>")
            .unwrap();
        let timeline = format!("{timeline_start}<aside id=\"timeline\">{tree}</aside>{timeline_end}");
        let featured = format!("{featured_start}<section id=\"featured\">{featured_list}</section>");
        let mut root = File::create(format!("{outdir}/index.html"))?;
        root.write(format!("{featured}{timeline}").as_bytes())?;
        let mut skeleton = File::open(format!("{outdir}/skeleton.html"))?;
        let mut buf = vec![];
        skeleton.read_to_end(&mut buf)?;
        let string = String::from_utf8(buf).unwrap();
        let (before, after) = string.split_once("<article></article>").unwrap();
        let article = html_string.to_string();
        let joined = format!("{before}<article id=\"post\">{article}</article>{after}");

        // write file out
        let date_for_filename = time.format("%Y-%b").to_string();
        let (year, month) = date_for_filename.split_once("-").unwrap();
        let postname = path.file_stem().unwrap().to_str().unwrap();
        let outdir_tagged = format!("{outdir}/posts/{year}/{month}");
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
