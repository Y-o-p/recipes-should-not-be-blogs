pub mod recipe;
pub mod webscraper;
pub mod markdown;

use std::{
    env,
    fs::File,
    io::prelude::*,
    collections::HashMap,
    error::Error,
};
use dotenvy::dotenv;

use clap::Parser;
use url::Url;

use webscraper::Website;
use recipe::Recipe;
use markdown::Markdown;

#[derive(Parser)]
struct Args {
    url: String,
    
    #[arg(short, long, default_value_t = String::from("./"))]
    output_path: String,

    #[arg(short, long, default_value_t = false)]
    website: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Get command line arguments
    let args = Args::parse();
    
    // Get environment variables
    dotenv().unwrap();
    
    let url: Url = Url::parse(args.url.as_str()).expect("Faulty URL");
    let mut recipe_website: Website = Website::from_scrape(url).await?;
    println!("Scraped website");
    recipe_website.regex_remove(r#"(comments)|(COMMENTS)|(Comments)[\s\S]*"#);
    println!("Removed comments");
    let recipe: Recipe = Recipe::from_get(recipe_website).await?;

    let mut recipe_text = recipe.as_markdown();
    println!("Converted text to markdown");

    if args.website {
        let date: String = format!("{}", recipe.date.format("%Y-%m-%d"));
        
        let header: String = format!(
            "+++\n\
            title = \"{}\"\n\
            template = \"page.html\"\n\
            date = {}\n\
            +++\n", 
            recipe.title,
            date
        );

        recipe_text.insert_str(0, header.as_str());
    }

    let mut full_path: String = String::from(args.output_path.as_str());
    full_path.push_str(format!("{}.md", recipe.title.as_str()).as_str());

    let mut recipe_file = File::create(&full_path.as_str())?;
    recipe_file.write_all(recipe_text.as_bytes());

    println!("Wrote recipe to: {}", full_path);

    let mut website_file = File::create("output/website.txt")?;
    website_file.write_all(recipe.website.plaintext.as_bytes());

    let mut chatgpt_file = File::create("output/chatgptresponse.txt")?;
    chatgpt_file.write_all(recipe.chatgpt_response.as_bytes());

    Ok(())
}
