#[allow(deprecated)]
use chrono::prelude::*;
use regex::Regex;
use std::env::args;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

mod helper;
mod parser;
mod timeline;

use helper::{extract_excerpt, list_directories};

fn get_recent(dir: &str) -> std::io::Result<String> {
    let mut string = "".to_string();
    let mut count = 0;
    let years = list_directories(dir)?;
    'main_loop: for year in years {
        let year = Path::new(&year);
        if year.is_dir() {
            let months = list_directories(year.to_str().unwrap())?;
            for month in months {
                let month = Path::new(&month);
                let entries = list_directories(month.to_str().unwrap())?;
                for entry in entries {
                    if count >= 3 {
                        break 'main_loop;
                    }
                    string += extract_excerpt(entry.as_str())?.as_str();
                    count += 1;
                }
            }
        }
    }
    Ok(string)
}

fn create_homepage(outdir: &String) -> std::io::Result<()> {
    let outdir_path = Path::new(outdir);
    let posts_dir = outdir_path.join("posts");
    let timeline = timeline::create_timeline(posts_dir.to_str().unwrap())?;
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
    let featured = format!("{featured_start}<section id=\"featured\">{featured_list}</section>");
    let timeline =
        format!("{timeline_start}<aside id=\"timeline\">{timeline}</aside>{timeline_end}");
    let mut root = File::create(format!("{outdir}/index.html"))?;
    root.write(format!("{featured}{timeline}").as_bytes())?;
    Ok(())
}

fn create_post_page(markdown_filepath: &String, outdir: &String) -> std::io::Result<()> {
    let path = Path::new(markdown_filepath);
    let mut file = File::open(path)?;
    let mut buf = vec![];
    file.read_to_end(&mut buf)?;
    let parsed_article = parser::to_html(buf);

    let time = Utc::now();

    let mut post_skeleton = File::open(format!("{outdir}/skeleton.html"))?;
    let mut buf = String::new();
    post_skeleton.read_to_string(&mut buf)?;

    let (title_before, title_after) = buf.split_once("<title></title>").unwrap();
    let (article_before, article_after) = title_after.split_once("<article></article>").unwrap();
    let regex = Regex::new(r".*?<h1><a.*?</a>(?<title>.*)?</h1>.*").unwrap();
    let captures = regex.captures(parsed_article.as_str()).unwrap();
    let title = &captures["title"];
    let joined =
        format!("{title_before}<title>{title}</title>{article_before}<article id=\"post\">{parsed_article}</article>{article_after}");

    // write file out
    let postname = path.file_stem().unwrap().to_str().unwrap();
    let (year, month, day) = (time.year(), time.month(), time.day());
    let outdir_tagged = format!("{outdir}/posts/{year}/{month}");
    std::fs::create_dir_all(&outdir_tagged)?;
    let outfilepath = format!("{outdir_tagged}/{day}_{postname}.html");
    let mut outfile = File::create(&outfilepath)?;
    outfile.write_all(joined.as_bytes())?;
    Ok(())
}

#[allow(dead_code)]
fn regen_site() {
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
        let filename = args.get(1).unwrap();
        let outdir = args.get(2).unwrap();
        if filename != "regen" {
            create_post_page(filename, outdir)?;
        }
        create_homepage(outdir)?;
        Ok(())
    }
}
