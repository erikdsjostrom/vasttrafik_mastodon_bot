#![deny(
    missing_docs,
    missing_crate_level_docs,
    missing_doc_code_examples,
    missing_debug_implementations
)]
//! News alert bot for västtrafik

mod config;
mod news;

use env_logger::{Env, Builder};
use log::info;
use anyhow::{bail, Result};
use elefren::{status_builder::Visibility, Language, Mastodon, MastodonClient, StatusBuilder};

use crate::config::Config;

/// Post a toot
/// Returns the id of the toot
fn toot(masto: &Mastodon, status_msg: String) -> Result<String> {
    let status = StatusBuilder::new()
        .status(status_msg)
        .visibility(Visibility::Public)
        .language(Language::Swe)
        .content_type("text/html")
        .build();
    let status = match status {
        Ok(status) => status,
        Err(e) => bail!("Error building status: {}", e),
    };
    match masto.new_status(status) {
        Ok(s) => Ok(s.id),
        Err(e) => bail!("Error posting status: {}", e),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    Builder::from_env(Env::default().default_filter_or("info")).init();
    info!("Starting up.");
    let mut config: Config = config::Config::new()?;
    let masto = Mastodon::from(config.mastodon.clone());
    let account = match masto.verify_credentials() {
        Ok(account) => account,
        Err(e) => bail!("Error verifying credentials: {}", e),
    };
    // If bot has never posted before, post everything.
    if account.statuses_count == 0 {
        info!("Bot has never posted before, posting all news.");
        let news = news::fetch().await?;
        for n in news.into_iter().rev() {
            let status_msg = n.to_string();
            let id = toot(&masto, status_msg)?;
            config.set_last_status_id(id);
            config.save()?;
        }
    }
    // Get title of latest toot
    let title = {
        let last_status = masto
            .get_status(&config.get_last_status_id())
            .unwrap();
        let last_status = scraper::html::Html::parse_document(&last_status.content);
        last_status
            .select(&scraper::Selector::parse("p").unwrap())
            .next()
            .unwrap()
            .inner_html()
    };
    let mut latest_news = news::latest().await?;
    // If latest news is not the same as the latest toot, post it.
    if latest_news.title != title {
        info!("Latest news is not the same as the latest toot, posting it.");
        let status_msg = latest_news.to_string();
        let id = toot(&masto, status_msg)?;
        config.set_last_status_id(id);
        config.save()?;
    }
    info!("Starting news fetching loop.");
    loop {
        let maybe_latest_news = news::latest().await?;
        if latest_news.title != maybe_latest_news.title {
            info!("New news, posting it.");
            latest_news = maybe_latest_news;
            let status_msg = latest_news.to_string();
            let id = toot(&masto, status_msg)?;
            config.set_last_status_id(id);
            config.save()?;
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(60 * 10)).await;
    }
}
