use chrono::NaiveDateTime;
use pulldown_cmark::{html, Parser};
use std::fs;
use std::io::Write;
use std::collections::BTreeMap;
use regex::Regex;
use gray_matter::Matter;
use gray_matter::engine::TOML;

const BLOG_SHELL_PATH: &str = "/Users/garyrob/src/zola/section3/content/blog.md";
const BLOG_DIR_PATH: &str = "/Users/garyrob/src/zola/section3/content/blog_content";

fn replace_posts_in_template(new_posts: &str) {
    let content = fs::read_to_string(BLOG_SHELL_PATH)
        .expect(&format!("Error reading {}", BLOG_SHELL_PATH));

    let re = Regex::new(r"(?s)(//\$\$begin posts\$\$\n)(.*?)(\n//\$\$end posts\$\$)")
        .expect("Failed to create the regex");

    let new_content = re.replace(&content, |caps: &regex::Captures| {
        format!("{}{}{}", &caps[1], new_posts, &caps[3])
    });

    let mut file = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(BLOG_SHELL_PATH)
        .expect(&format!("Error opening {}", BLOG_SHELL_PATH));

    file.write_all(new_content.as_bytes())
        .expect(&format!("Error writing to {}", BLOG_SHELL_PATH));
}

fn remove_front_matter(post_content: &str) -> String {
    let re_front_matter = Regex::new(r"(?s)^\+\+\+.*?\+\+\+\n").unwrap();
    re_front_matter.replace(post_content, "").to_string()
}

fn main() {
    let mut entries: BTreeMap<NaiveDateTime, (String, String)> = BTreeMap::new();
    let re = Regex::new(r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}\.md$").expect("Failed to compile regex");

    for entry in fs::read_dir(BLOG_DIR_PATH).expect("Failed to read directory") {
        if let Ok(entry) = entry {
            println!("unfiltered file");
            let path = entry.path();
            let file_name_str = path.file_name().unwrap().to_string_lossy();
            println!("file_name_str: {}",file_name_str);
            if re.is_match(&file_name_str) {
                println!("filtered file {}", &file_name_str);
                let content = fs::read_to_string(&path).expect("Failed to read file");
                println!("Content: {}", &content);
                let mut matter = Matter::<TOML>::new();
                matter.delimiter = "+++".to_owned();
                let something = matter.parse(&content);

                let file_stem = path.file_stem().unwrap().to_str().unwrap();
                println!("File stem: {}",file_stem);
                match chrono::NaiveDateTime::parse_from_str(file_stem, "%Y-%m-%d %H:%M") {
                    Ok(datetime) => {
                        match something.data.as_ref().unwrap()["title"].as_string() {
                            Ok(title) => {
                                println!("title {}", &title);
                                let markdown_content = content.trim();
                                let parser = Parser::new(&markdown_content);
                                let mut html_content = String::new();
                                html::push_html(&mut html_content, parser);
                                let word_count = html_content.split_whitespace().count();
                                let read_time = (word_count / 200) + 1;
                                let mut excerpt = markdown_content.split_whitespace().take(100).collect::<Vec<&str>>().join(" ");
                                if word_count > 100 {
                                    excerpt.push_str(&format!("... [more]({})", path.display()));
                                }
                                entries.insert(datetime, (
                                    title.to_string(),
                                    format!(
                                        "<h3>{}</h3>\n<small>{} - {} words - {} mins</small>\n\n{}",
                                        title, datetime, word_count, read_time, excerpt
                                    )
                                ));
                                println!("#####################did insert: {}", datetime);
                            },
                            Err(e) => println!("Failed to parse front matter: {}", e),
                        }
                    },
                    Err(_) => {
                        println!("Error: The file '{}' has an invalid name format.", file_stem);
                    }
                }
            }
        }
    }

    let mut output = String::new();
    for (_, (_, content)) in entries {
        let content_without_front_matter = remove_front_matter(&content);
        output.push_str(&format!("\n{}\n", content_without_front_matter));
        println!("one entry: {}", content_without_front_matter);
    }

    replace_posts_in_template(&output);
}
