use chrono::NaiveDateTime;
use pulldown_cmark::{Parser, Event};
use std::fs;
use std::io::Write;
use std::collections::BTreeMap;
use std::path::Path;
use std::process;
use regex::Regex;
use gray_matter::Matter;
use gray_matter::engine::TOML;

const BLOG_SHELL_PATH: &str = "/Users/garyrob/src/zola/section3/content/blog.md";
const BLOG_DIR_PATH: &str = "/Users/garyrob/src/zola/section3/content/blog_content";
const BLOG_POSTS_PATH: &str = "/Users/garyrob/src/zola/section3/content/posts";

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
    // Change the format to use an underscore instead of space
    chrono::NaiveDateTime::parse_from_str(file_stem, "%Y-%m-%dt%H-%M")
        .map_err(|_| format!("Error: The file '{}' does not have a proper date-time format.", file_stem))
}

fn get_title(raw_content: &str) -> String {
    let mut matter = Matter::<TOML>::new();
    matter.delimiter = "+++".to_owned();
    let parsed_content = matter.parse(raw_content);

    if let Some(title) = parsed_content.data.as_ref().unwrap()["title"].as_string().ok() {
        title.to_string()
    } else {
        eprintln!("ERROR: Failed to parse front matter for title. ABORTING.");
        process::exit(1);
    }
}

fn get_nicer_blog_content(less_nice: &str) -> String {
    let title = get_title(less_nice);
    if title.contains("#") {
        eprintln!("ERROR: Title contains '#'. ABORTING.");
        std::process::exit(1);        
    }
    let added_heading_level = less_nice.replace("# ", "## ");
    let parts: Vec<&str> = added_heading_level.splitn(3, "+++").collect();
    if parts.len() < 3 {
        eprintln!("ERROR: Input does not contain two occurrences of '+++'. ABORTING.");
        std::process::exit(1);
    }
    if !parts[0].is_empty() {
        eprintln!("ERROR: Unexpected content before first '+++'. ABORTING.");
        std::process::exit(1);
    }
    format!("+++{}+++\n# {}{}", parts[1], title, parts[2])
}

fn write_post(datetime: &str, content: &str) {
    let path = Path::new(BLOG_POSTS_PATH).join(datetime);
    if fs::write(path, content).is_err() {
        eprintln!("ERROR: Failed to write content to file. ABORTING.");
        std::process::exit(1);
    }
}

fn get_without_headers(content: &str) -> String {
    content.lines()
        .map(|line| {
            if line.starts_with('#') {
                let trimmed_line = line.trim_start_matches('#').trim();
                format!("{}:", trimmed_line)
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<String>>()
        .join("\n")
}

fn remove_markdown(markdown: &str) -> String {
    let parser = Parser::new(markdown);
    let mut text = String::new();
    for event in parser {
        match event {
            Event::Text(t) => text.push_str(&t),
            Event::Code(c) => text.push_str(&c),
            Event::SoftBreak | Event::HardBreak => text.push('\n'),
            _ => (),
        }
    }
    text
}

fn make_entry_summary_html(datetime: &NaiveDateTime, raw_content: &str, file_stem: &str) -> Result<String, String> {
    let content_no_frontmatter = remove_front_matter(raw_content);
    let content_no_frontmatter_or_headers = get_without_headers(&content_no_frontmatter);
    // I removed headers in a way that puts a colon at the end of the header
    // line. Now I try to remove the rest of the markdown.
    let content_pure_text = remove_markdown(&content_no_frontmatter_or_headers);
    let title = get_title(raw_content);
   
    let word_count = content_pure_text.split_whitespace().count();
    let read_time = (word_count / 200) + 1;
    
    let css_id = title.to_lowercase().replace(" ", "-");
    let style = format!(
        "<style>\n,<h1>{}</h1> + p {{\n    margin-top: -20px; /* Adjust as necessary */\n}}\n</style>\n",
        css_id
    );

    
    // Truncate if more than 100 words
    let display_content = if word_count > 100 {
        let excerpt = content_pure_text.split_whitespace().take(100).collect::<Vec<&str>>().join(" ");
        format!("{}... <a href=\"/posts/{}\">more</a>", excerpt, file_stem)
    } else {
        content_pure_text
    };

    let linked_title = format!("<a href=\"/posts/{}\">{}</a>", file_stem, title);

    let formatted_entry_summary = format!(
        "{}\n# {}\n<small>{} - {} words - {} mins</small>\n\n{}<br>",
        style, linked_title, datetime, word_count, read_time, display_content
    );
    
    Ok(formatted_entry_summary)
}



fn process_next_summary_entry_content(datetime: NaiveDateTime, raw_content: &str, file_stem: &str) -> Result<(NaiveDateTime, String), String> {
    match make_entry_summary_html(&datetime, raw_content, file_stem) {
        Ok(entry_summary_html) => Ok((datetime, entry_summary_html)),
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
                    
                    match process_next_summary_entry_content(datetime, &raw_content, file_stem) {
                        Ok((datetime, entry_summary_html)) => {
                            entries.insert(datetime, entry_summary_html);
                        },
                        Err(err_msg) => {
                            println!("{}", err_msg);
                        }
                    }
                    let post_content = get_nicer_blog_content(&raw_content);
                    let post_file_name = format!("{}.md", file_stem);
                    write_post(&post_file_name, &post_content);
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