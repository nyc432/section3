use chrono::NaiveDate;
use pulldown_cmark::{html, Parser};
use std::fs;
use std::collections::BTreeMap;
use regex::Regex;
use gray_matter::Matter;
use gray_matter::engine::TOML;
use serde::Deserialize;

const INPUT: &str = r#"---
title: gray-matter-rs
tags:
  - gray-matter
  - rust
---
Some excerpt
---
Other stuff
"#;

fn main() {
    let blog_dir = "/Users/garyrob/src/zola/section3/content/blog_content";
    let mut entries: BTreeMap<NaiveDate, (String, String)> = BTreeMap::new();
    let re = Regex::new(r"^\d{4}-\d{2}-\d{2}\.md$").expect("Failed to compile regex");

    for entry in fs::read_dir(blog_dir).expect("Failed to read directory") {
        if let Ok(entry) = entry {
            println!("unfiltered file");
            let path = entry.path();
            let file_name_str = path.file_name().unwrap().to_string_lossy();

            if re.is_match(&file_name_str) {
                println!("filtered file {}", &file_name_str);
                let content = fs::read_to_string(&path).expect("Failed to read file");
                println!("Content: {}", &content);
                let mut matter = Matter::<TOML>::new();
                matter.delimiter = "+++".to_owned();
                let something = matter.parse(&content);
                // println!("######New: {:#?}", something.data.as_ref().unwrap()["title"].as_string());
                match something.data.as_ref().unwrap()["title"].as_string() {
                    Ok(title) => {
                        println!("title {}", &title);
                        let date = chrono::NaiveDate::parse_from_str(path.file_stem().unwrap().to_str().unwrap(), "%Y-%m-%d").expect("Failed to parse date");
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
                        entries.insert(date, (
                            title.to_string(),
                            format!(
                                "<h3>{}</h3>\n<small>{} - {} words - {} mins</small>\n\n{}",
                                title, date, word_count, read_time, excerpt
                            )
                        ));
                            
                        
                    },
                    Err(e) => println!("Failed to parse front matter: {}", e),
                }
            }
        }
    }

    let mut output = String::new();
    for (_, (_, content)) in entries {
        output.push_str(&format!("\n{}\n", content));
        println!("one entry: {}", content);
    }

    std::fs::write("/Users/garyrob/src/zola/section3/test_blog.md", output).expect("Failed to write output");

   


}
