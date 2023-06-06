use url::Url;
use reqwest::Client;
use std::error::Error;
use regex::Regex;

pub struct Website {
    pub url: String,
    raw_html: String,
    pub plaintext: String
}

impl Website {
    pub fn new() -> Website {
        Website {
            url: String::new(),
            raw_html: String::new(),
            plaintext: String::new()
        }
    }

    pub async fn from_scrape(url: Url) -> Result<Website, Box<dyn Error>> {
        let mut website = Website::new();
        website.scrape(url).await?;
        Ok(website)
    }

    pub fn remove_html_and_links(data: &String) -> String {
        let reg = Regex::new(r#"<[^<]*>|\[.*\]|(https?:\/\/(www\.)?)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_\+.~#?&//=]*)|[@#$%^&*\[\]\(\)\\=+_\|]"#).unwrap();
        reg.replace_all(&data, "").to_string()
    }

    pub async fn scrape(&mut self, url: Url) -> Result<(), Box<dyn Error>> {
        let client = Client::new();
        self.url = String::from(url.as_str());

        // Get the HTML
        self.raw_html = client.get(url)
            .send()
            .await?
            .text()
            .await?;

        // Get the plaintext
        let mut toolsyep_url = Url::parse_with_params("https://toolsyep.com/en/webpage-to-plain-text/",
                                                    &[("u", self.url.as_str())])?;
        let html = client.get(toolsyep_url)
            .send()
            .await?
            .text()
            .await?;
        self.plaintext = Website::remove_html_and_links(&html);

        Ok(())
    }
}