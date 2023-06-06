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
    
    #[arg(short, long, default_value_t = String::from("./recipe.md"))]
    output_path: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Get command line arguments
    let args = Args::parse();
    
    // Get environment variables
    dotenv().unwrap();
    
    let url: Url = Url::parse(args.url.as_str()).expect("Faulty URL");
    let recipe_website: Website = Website::from_scrape(url).await?;
    let recipe: Recipe = Recipe::from_get(recipe_website).await?;
    
    let mut recipe_file = File::create(&args.output_path)?;
    recipe_file.write_all(recipe.as_markdown().as_bytes());

    let mut website_file = File::create("website")?;
    website_file.write_all(recipe.website.plaintext.as_bytes());

    let mut chatgpt_file = File::create("chatgptresponse")?;
    chatgpt_file.write_all(recipe.chatgpt_response.as_bytes());

    Ok(())
}
