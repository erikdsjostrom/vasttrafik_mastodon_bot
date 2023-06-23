use anyhow::{bail, Result};
use scraper::{ElementRef, Html, Selector};

const BASE: &str = "https://www.vasttrafik.se";
const ARKIV: &str = "/om-vasttrafik/nyhetsarkiv/";

/// News struct
#[derive(Debug, Clone)]
pub struct News {
    pub title: String,
    pub body: String,
    pub url: String,
}

impl From<&ElementRef<'_>> for News {
    fn from(value: &ElementRef) -> Self {
        let title = parse_news_title(value).unwrap();
        let body = parse_news_body(value).unwrap();
        let url = get_news_page(value).unwrap();
        Self { title, body, url }
    }
}

impl std::fmt::Display for News {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n\n{}\nLäs mer: {}", self.title, self.body, self.url)
    }
}

/// Fetches the news from västtrafik
pub async fn fetch() -> Result<Vec<News>> {
    let url = format!("{}{}", BASE, ARKIV);
    let body = reqwest::get(url).await?.text().await?;
    let content = Html::parse_document(&body);
    parse_news_blocks(&content).map(|news_blocks| {
        news_blocks
            .iter()
            .map(News::from)
            .collect::<Vec<_>>()
    })
}

/// Fetches the latests news item from västtrafik
pub async fn latest() -> Result<News> {
    let news = fetch().await?;
    match news.first() {
        Some(n) => Ok(n.clone()),
        None => bail!("No news found"),
    }
}

/// Get the link to the news page
fn get_news_page(news_block: &ElementRef) -> Result<String> {
    if let Some(href) = news_block.value().attr("href") {
        let url = format!("{}{}", BASE, href);
        Ok(url)
    } else {
        bail!("No href found");
    }
}

/// Parses out the news blocks
fn parse_news_blocks(html: &Html) -> Result<Vec<ElementRef>> {
    let selector = Selector::parse("#news-list > a").unwrap();
    let news_blocks = html.select(&selector).collect::<Vec<_>>();
    if news_blocks.is_empty() {
        bail!("No news blocks found");
    }
    Ok(news_blocks)
}

/// Parses out the title of the news block
fn parse_news_title(news_block: &ElementRef) -> Result<String> {
    let selector = Selector::parse(".news-list-page__news-heading").unwrap();
    let news_title = news_block.select(&selector).collect::<Vec<_>>();
    if news_title.is_empty() {
        bail!("No news title found");
    }
    Ok(news_title[0].text().collect::<String>().trim().to_string())
}

/// Parses out the body of the news block
fn parse_news_body(news_block: &ElementRef) -> Result<String> {
    let selector = Selector::parse(".news-list-page__news-introduction").unwrap();
    let news_body = news_block.select(&selector).collect::<Vec<_>>();
    if news_body.is_empty() {
        bail!("No news body found");
    }
    Ok(news_body[0].text().collect::<String>().trim().to_string())
}
