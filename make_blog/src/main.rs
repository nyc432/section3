use chrono::NaiveDateTime;
use pulldown_cmark::{html, Parser};
use std::fs;
use std::io::Write;
use std::collections::BTreeMap;
use std::process;
use regex::Regex;
use gray_matter::Matter;
use gray_matter::engine::TOML;

const BLOG_SHELL_PATH: &str = "/Users/garyrob/src/zola/section3/content/blog.md";
const BLOG_DIR_PATH: &str = "/Users/garyrob/src/zola/section3/content/blog_content";

fn replace_posts_in_template(new_posts: &str) {
    println!("XXXXXXXX Content to Insert:\n{}", new_posts);
    let content = fs::read_to_string(BLOG_SHELL_PATH)
        .expect(&format!("Error reading {}", BLOG_SHELL_PATH));

    let re = Regex::new(r"(?s)(<!--\$\$begin posts\$\$-->\n)(.*?)(\n<!--\$\$end posts\$\$-->)")
        .expect("Failed to create the regex");

    let new_content = re.replace(&content, |caps: &regex::Captures| {
        format!("{}{}{}", &caps[1], new_posts, &caps[3])
    });

    // Check if replacement was successful
    if re.captures(&content).is_none() {
        eprintln!("ERROR: FAILED TO FIND AND REPLACE CONTENT BETWEEN MARKERS. ABORTING.");
        process::exit(1);
    }

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
    re_front_matter.replace_all(post_content, "").trim().to_string()
}

fn get_datetime(file_stem: &str) -> Result<NaiveDateTime, String> {
    chrono::NaiveDateTime::parse_from_str(file_stem, "%Y-%m-%d %H:%M")
        .map_err(|_| format!("Error: The file '{}' does not have a proper date-time format.", file_stem))
}

fn make_full_content(datetime: &NaiveDateTime, raw_content: &str) -> Result<String, String> {
    let content = remove_front_matter(raw_content);
    let mut matter = Matter::<TOML>::new();
    matter.delimiter = "+++".to_owned();
    let parsed_content = matter.parse(raw_content);
    
    if let Some(title) = parsed_content.data.as_ref().unwrap()["title"].as_string().ok() {
        let parser = Parser::new(&content);
        let mut html_content = String::new();
        html::push_html(&mut html_content, parser);
        
        let word_count = html_content.split_whitespace().count();
        let read_time = (word_count / 200) + 1;
        
        let css_id = title.to_lowercase().replace(" ", "-");
        let style = format!(
            "<style>\nh1#{} + p {{\n    margin-top: -20px; /* Adjust as necessary */\n}}\n</style>\n",
            css_id
        );
        
        // Truncate if more than 100 words
        let mut display_content = html_content.clone();
        if word_count > 100 {
            let excerpt = html_content.split_whitespace().take(100).collect::<Vec<&str>>().join(" ");
            display_content = format!("{}... [more]", excerpt);
        }
        
        let formatted_content = format!(
            "{}\n# {}\n<small>{} - {} words - {} mins</small>\n\n{}<br>",
            style, title, datetime, word_count, read_time, display_content
        );
        
        Ok(formatted_content)
    } else {
        Err(format!("Failed to parse front matter for: {}", datetime))
    }
}

fn process_content(datetime: NaiveDateTime, raw_content: &str) -> Result<(NaiveDateTime, String), String> {
    match make_full_content(&datetime, raw_content) {
        Ok(full_content) => Ok((datetime, full_content)),
        Err(err_msg) => Err(err_msg),
    }
}


fn main() {
    let mut entries: BTreeMap<NaiveDateTime, String> = BTreeMap::new();

    for entry in fs::read_dir(BLOG_DIR_PATH).expect("Failed to read directory") {
        if let Ok(entry) = entry {
            let path = entry.path();
            let file_stem = path.file_stem().unwrap().to_str().unwrap();

            match get_datetime(file_stem) {
                Ok(datetime) => {
                    let raw_content = fs::read_to_string(&path).expect("Failed to read file");
                    
                    match process_content(datetime, &raw_content) {
                        Ok((datetime, full_content)) => {
                            entries.insert(datetime, full_content);
                        },
                        Err(err_msg) => {
                            println!("{}", err_msg);
                        }
                    }
                },
                Err(err_msg) => {
                    println!("{}", err_msg);
                }
            }
        }
    }

    let mut output = String::new();
    for (_, content) in entries {
        output.push_str(&format!("\n{}\n", content)); 
        println!("\n-------------------------------");
        println!("{}", content);
        println!("-------------------------------\n");
    }

    replace_posts_in_template(&output);
}