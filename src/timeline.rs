use std::path::Path;

use crate::helper::{get_title, list_directories};

static NAMES: [&'static str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

pub fn create_timeline(path: &str) -> std::io::Result<String> {
    let mut list_html = String::new();
    let years = list_directories(path)?;
    for year in years {
        let year = Path::new(&year);
        if year.is_dir() {
            list_html += format!(
                "<li><details>\n\t<summary><span><h2>{}</h2></span></summary>\n\t<ul>\n",
                year.file_stem().unwrap().to_string_lossy()
            )
            .as_str();
            let months = list_directories(year.to_str().unwrap())?;
            for month in months {
                let month = Path::new(&month);
                list_html += format!(
                    "\t\t<li><details>\n\t\t\t<summary><span><h3>{}</h3></span></summary>\n\t\t\t<ul>\n",
                    NAMES[usize::from_str_radix(month.file_stem().unwrap().to_str().unwrap(), 10)
                        .unwrap()]
                )
                .as_str();
                let entries = list_directories(month.to_str().unwrap())?;
                for entry in entries {
                    let entry = Path::new(&entry);
                    let href = format!(
                        "/blog/posts/{}/{}/{}",
                        year.file_stem().unwrap().to_string_lossy(),
                        month.file_stem().unwrap().to_string_lossy(),
                        entry.file_name().unwrap().to_string_lossy()
                    );
                    let title = get_title(entry);
                    list_html +=
                        format!("\t\t\t\t<li><a href=\"{href}\"><h4>{title}</h4></a></li>\n")
                            .as_str();
                }
                list_html += "\t\t\t</ul>\n\t\t</details></li>\n";
            }
            list_html += "\t</ul>\n</details></li>\n";
        }
    }
    let html = format!("<h1>Timeline</h1>\n<ul>\n{list_html}</ul>");
    Ok(html)
}
