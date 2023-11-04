use regex::RegexBuilder;
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn list_directories(path: &str) -> std::io::Result<Vec<String>> {
    let path = Path::new(path);
    let pathlist = path
        .read_dir()?
        .map(|x| x.map(|x| x.path()))
        .collect::<Result<Vec<_>, std::io::Error>>()?;

    let re = regex::Regex::new(r".*.DS_Store.*").unwrap();
    let pathlist = pathlist
        .iter()
        .filter(|x| !re.is_match(x.to_str().unwrap()))
        .collect::<Vec<_>>();
    let mut pathlist = pathlist
        .iter()
        .map(|x| String::from(x.to_str().unwrap()))
        .collect::<Vec<_>>();

    pathlist.sort_by(|x, y| {
        let x = Path::new(x).file_stem().unwrap().to_str().unwrap();
        let y = Path::new(y).file_stem().unwrap().to_str().unwrap();
        if let Ok(num_x) = i32::from_str_radix(x, 10) {
            let num_y = i32::from_str_radix(y, 10).unwrap();
            num_y.cmp(&num_x)
        } else {
            let (x, _) = x.split_once("_").unwrap();
            let (y, _) = y.split_once("_").unwrap();
            i32::from_str_radix(y, 10)
                .unwrap()
                .cmp(&i32::from_str_radix(x, 10).unwrap())
        }
    });

    Ok(pathlist)
}

pub fn extract_excerpt(path: &str) -> std::io::Result<String> {
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

pub fn get_title(path: &Path) -> String {
    let mut file = File::open(path).unwrap();
    let mut buf = String::new();
    file.read_to_string(&mut buf).unwrap();
    let regex = regex::Regex::new(r".*<title>(?<title>.*)</title>.*").unwrap();
    let captures = regex.captures(&buf).unwrap();
    return String::from(&captures["title"]);
}
