// https://api-ninjas.com/api/nutrition

use reqwest::header::{
    HeaderMap,
    HeaderValue,
};
use serde::{Serialize, Deserialize};
use serde_json::{Value};
use regex::Regex;
use html2text;
use std::{
    env,
    fs,
    fs::File,
    io::prelude::*,
    iter::repeat,
    collections::HashMap,
};
use dotenvy::dotenv;
use openai::{
    chat::{ChatCompletion, ChatCompletionMessage, ChatCompletionMessageRole},
    set_key,
};

const FOOD_NUM_ELEMENTS: usize = 12;

#[derive(Serialize, Deserialize)]
struct Food {
    food_name: String,
    brand_name: Value,
    serving_qty: f32,
    serving_unit: String,
    serving_weight_grams: f32,
    nf_calories: f32,
    nf_total_fat: f32,
    nf_saturated_fat: f32,
    nf_cholesterol: f32,
    nf_sodium: f32,
    nf_total_carbohydrate: f32,
    nf_dietary_fiber: f32,
    nf_sugars: f32,
    nf_protein: f32,
    nf_potassium: f32,
    nf_p: f32,
    full_nutrients: Vec<Nutrients>,
    nix_brand_name: Value,
    nix_brand_id: Value,
    nix_item_name: Value,
    nix_item_id: Value,
    upc: Value,
    consumed_at: Value,
    metadata: Value,
    source: u32,
    ndb_no: u32,
    tags: Value,
    alt_measures: Value,
    lat: Value,
    lng: Value,
    meal_type: u32,
    photo: Value,
    sub_recipe: Value,
    class_code: Value,
    brick_code: Value,
    tag_id: Value,
}

#[derive(Serialize, Deserialize)]
struct NutritionResponse {
    foods: Vec<Food>,
}

#[derive(Serialize, Deserialize)]
struct Nutrients {
    attr_id: u32,
    value: f32
}

fn remove_html(text: String) {
    //let plain_text = html2text::from_read(text.as_bytes(), 100).to_string();
    let reg = Regex::new(r#"<[^<]*>|\[.*\]|(https?:\/\/(www\.)?)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_\+.~#?&//=]*)|[@#$%^&*\[\]\(\)\\=+_\|]"#).unwrap();
    reg.replace_all(&text, "");
}

async fn get_directions_and_ingredients(recipe: String) -> String {
    // Prompt ChatGPT to get the directions and ingredients
    let user_message_content = format!("Take the following recipe and give me just the title, number of servings, list of ingredients, and then the directions:\n{}", recipe);
    let message = ChatCompletionMessage {
        role: ChatCompletionMessageRole::User,
        content: user_message_content,
        name: None,
    };
    let chat_completion = ChatCompletion::builder("gpt-3.5-turbo", vec!(message))
        .create()
        .await.unwrap().unwrap();
    
    chat_completion.choices.first()
        .unwrap()
        .message
        .clone()
        .content
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().unwrap();
    set_key(env::var("OPENAI_KEY").unwrap());
    set_key(env::var("API_NINJAS_KEY").unwrap());
    let client = reqwest::Client::new();

    let html = client.get("https://www.delish.com/cooking/recipe-ideas/a19636089/creamy-tuscan-chicken-recipe/")
        .send()
        .await?
        .text()
        .await?;
        
    let recipe = remove_html(html);

    let myfile = fs::read_to_string("text")?;
    //println!("{}", myfile);

    let mut new_file = File::create("newfile.md")?;
    
    // Get each of the ingredients
    let find_ingredients = Regex::new(r#"-.*"#).unwrap();
    let mut ingredients = Vec::new();
    for ingredient in find_ingredients.captures_iter(&myfile[..]) {
        ingredients.push(&ingredient.get(0).unwrap().as_str()[1..]);
    }

    // Get nutritional value of each ingredient
    let mut ingredients_as_str = String::new();
    for ingredient in ingredients {
        ingredients_as_str.push_str(&ingredient[..]);
    }
    println!("{}", ingredients_as_str);
    let mut body = HashMap::new();
    body.insert("query", ingredients_as_str);
    let url_string = "https://trackapi.nutritionix.com/v2/natural/nutrients";
    let url = reqwest::Url::parse(&url_string)?;
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", HeaderValue::from_static("application/json"));
    let nutrition_app_id = env::var("NUTRITIONIX_APP_ID").expect("NUTRITIONIX_APP_ID hasn't been set...");
    let nutrition_app_key = env::var("NUTRITIONIX_APP_KEY").expect("NUTRITIONIX_APP_KEY hasn't been set...");
    headers.insert("x-app-id", HeaderValue::from_str(nutrition_app_id.as_str()).unwrap());
    headers.insert("x-app-key", HeaderValue::from_str(nutrition_app_key.as_str()).unwrap());
    let request_builder = client.post(url)
        .json(&body)
        .headers(headers);

    let response = request_builder.send().await?;
    
    //let mut data = &response.json::<NutritionResponse>().await?;
    let mut response_text: String = response.text().await?;
    //println!("{}", response_text.as_str());
    let data: NutritionResponse = serde_json::from_str(response_text.as_str())?;
    println!("{}", data.foods[0].food_name);
    
    // Build the table of ingredients and their nutrients
    let mut table_str: String = String::new();
    table_str.push_str("| INGREDIENT | CALORIES | PROTEIN | CARBS | FAT |\n");
    table_str.push_str("| - | - | - | - | - |\n");
    let mut total_calories: f32 = 0.0;
    let mut total_protein: f32 = 0.0;
    let mut total_carbs: f32 = 0.0;
    let mut total_fat: f32 = 0.0;
    for food in data.foods {
        table_str.push_str(&format!("| {} {} {} | {} | {} | {} | {} |\n", food.serving_qty, food.serving_unit, food.food_name, food.nf_calories, food.nf_protein, food.nf_total_carbohydrate, food.nf_total_fat)[..]);
        total_calories += food.nf_calories; 
        total_protein += food.nf_protein; 
        total_carbs += food.nf_total_carbohydrate; 
        total_fat += food.nf_total_fat;
    }
    new_file.write_all(table_str.as_bytes());
    
    // Build a table of the total nutrients
    let mut tot_table_str: String = String::new();
    tot_table_str.push_str("\n| MACRO | AMOUNT |\n");
    tot_table_str.push_str("| - | - |\n");
    tot_table_str.push_str(&format!("| TOTAL CALORIES | {} |\n", total_calories)[..]);
    tot_table_str.push_str(&format!("| TOTAL PROTEIN | {} |\n", total_protein)[..]);
    tot_table_str.push_str(&format!("| TOTAL CARBS | {} |\n", total_carbs)[..]);
    tot_table_str.push_str(&format!("| TOTAL FAT | {} |\n", total_fat)[..]);
    new_file.write_all(tot_table_str.as_bytes());

    // Get each direction
    let find_directions = Regex::new(r#"[1-9]\..*"#).unwrap();
    let mut directions = Vec::new();
    for direction in find_directions.captures_iter(&myfile[..]) {
        directions.push(direction.get(0).unwrap().as_str());
    }

    // Add the directions
    let mut directions_str: String = String::new();
    for direction in directions {
        directions_str.push_str(&format!("{}\n", direction)[..]);
    }
    new_file.write_all(directions_str.as_bytes());

    Ok(())
}
