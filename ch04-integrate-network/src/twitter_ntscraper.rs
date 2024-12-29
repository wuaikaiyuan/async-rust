
use reqwest::{Client, ClientBuilder};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use tokio;
use base64;
use chrono::{DateTime, NaiveDateTime, Utc};
use url::Url;
use std::time::Duration;
use anyhow::{Result, Context};
use futures::stream::{self, StreamExt};
use rand::seq::SliceRandom;
use log::{info, warn};

// Valid filters that can be applied to searches
const VALID_FILTERS: [&str; 11] = [
    "nativeretweets",
    "media",
    "videos",
    "news", 
    "verified",
    "native_video",
    "replies",
    "links",
    "images",
    "safe",
    "quote"
];

#[derive(Debug, Serialize, Deserialize)]
pub struct TweetStats {
    comments: i32,
    retweets: i32,
    quotes: i32,
    likes: i32
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    name: String,
    username: String,
    profile_id: String,
    avatar: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tweet {
    link: String,
    text: String,
    user: User,
    date: String,
    is_retweet: bool,
    is_pinned: bool,
    external_link: String,
    replying_to: Vec<String>,
    quoted_post: Option<QuotedTweet>,
    stats: TweetStats,
    pictures: Vec<String>,
    videos: Vec<String>,
    gifs: Vec<String>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuotedTweet {
    link: String,
    text: String,
    user: User,
    date: String,
    pictures: Vec<String>,
    videos: Vec<String>,
    gifs: Vec<String>
}

#[derive(Debug)]
pub struct NitterClient {
    client: Client,
    instances: Vec<String>,
    working_instances: Vec<String>,
    current_instance: String,
    retry_count: u32,
    cooldown_count: u32,
    session_reset: bool
}

impl NitterClient {
    pub async fn new(instances: Option<Vec<String>>, skip_instance_check: bool) -> Result<Self> {
        let client = ClientBuilder::new()
            .timeout(Duration::from_secs(10))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:129.0) Gecko/20100101 Firefox/129.0")
            .build()?;

        let instances = match instances {
            Some(i) => i,
            None => Self::get_instances(&client).await?
        };

        let mut nitter = NitterClient {
            client,
            instances: instances.clone(),
            working_instances: vec![],
            current_instance: String::new(),
            retry_count: 0,
            cooldown_count: 0,
            session_reset: false
        };

        if skip_instance_check {
            nitter.working_instances = instances;
        } else {
            nitter.test_all_instances("/x", true).await?;
        }

        if nitter.working_instances.is_empty() {
            anyhow::bail!("No working instances found");
        }

        nitter.current_instance = nitter.get_random_instance()?;
        Ok(nitter)
    }

    async fn get_instances(client: &Client) -> Result<Vec<String>> {
        let resp = client
            .get("https://raw.githubusercontent.com/libredirect/instances/main/data.json")
            .send()
            .await?;

        #[derive(Deserialize)]
        struct InstancesResponse {
            nitter: NitterInstances
        }

        #[derive(Deserialize)]
        struct NitterInstances {
            clearnet: Vec<String>
        }

        let instances: InstancesResponse = resp.json().await?;
        Ok(instances.nitter.clearnet)
    }

    pub fn get_random_instance(&self) -> Result<String> {
        self.working_instances
            .choose(&mut rand::thread_rng())
            .cloned()
            .context("No working instances available")
    }

    async fn test_all_instances(&mut self, endpoint: &str, silent: bool) -> Result<()> {
        if !silent {
            info!("Testing all instances...");
        }

        let mut working = Vec::new();

        for instance in &self.instances {
            match self.test_instance(instance, endpoint).await {
                Ok(true) => working.push(instance.clone()),
                _ => continue
            }
        }

        if !silent {
            info!("Found {} working instances", working.len());
        }

        self.working_instances = working;
        Ok(())
    }

    async fn test_instance(&self, instance: &str, endpoint: &str) -> Result<bool> {
        let url = format!("{}{}", instance, endpoint);
        
        match self.client.get(&url)
            .send()
            .await 
        {
            Ok(resp) => {
                if !resp.status().is_success() {
                    return Ok(false);
                }
                
                let html = resp.text().await?;
                let document = Html::parse_document(&html);
                
                let selector = Selector::parse("div.timeline-item").unwrap();
                Ok(document.select(&selector).next().is_some())
            }
            Err(_) => Ok(false)
        }
    }

    pub async fn get_tweets(
        &mut self,
        terms: Vec<String>,
        mode: &str,
        number: i32,
        since: Option<String>,
        until: Option<String>,
        near: Option<String>,
        language: Option<String>,
        to: Option<String>,
        replies: bool,
        filters: Option<Vec<String>>,
        exclude: Option<Vec<String>>,
        max_retries: u32
    ) -> Result<Vec<Tweet>> {
        let mut tweets = Vec::new();

        for term in terms {
            let endpoint = self.build_search_endpoint(
                &term,
                mode,
                since.as_deref(),
                until.as_deref(),
                near.as_deref(),
                language.as_deref(),
                to.as_deref(),
                replies,
                filters.as_ref(),
                exclude.as_ref()
            )?;

            let mut keep_scraping = true;
            let mut cursor = None;

            while keep_scraping {
                let page_tweets = self.fetch_page_tweets(&endpoint, cursor.as_deref()).await?;
                
                tweets.extend(page_tweets.iter().cloned());

                if tweets.len() >= number as usize {
                    tweets.truncate(number as usize);
                    break;
                }

                if let Some(next_cursor) = self.get_next_cursor(&endpoint).await? {
                    cursor = Some(next_cursor);
                } else {
                    keep_scraping = false;
                }

                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }

        Ok(tweets)
    }

    // Helper methods for parsing tweets from HTML
    fn parse_tweet(&self, element: scraper::ElementRef) -> Result<Tweet> {
        let tweet_selector = Selector::parse("div.tweet-content").unwrap();
        let text = element
            .select(&tweet_selector)
            .next()
            .map(|el| el.text().collect::<String>())
            .unwrap_or_default();

        // Similar parsing for other tweet fields...
        // This would include parsing the user info, stats, media, etc.
        
        Ok(Tweet {
            text,
            // Other fields would be populated here
            ..Default::default()
        })
    }

    async fn extract_media(&self, element: scraper::ElementRef, is_encrypted: bool) -> Result<(Vec<String>, Vec<String>, Vec<String>)> {
        let mut pictures = Vec::new();
        let mut videos = Vec::new();
        let mut gifs = Vec::new();

        let attachments_selector = Selector::parse("div.attachments").unwrap();
        if let Some(attachments) = element.select(&attachments_selector).next() {
            // Extract pictures
            let img_selector = Selector::parse("img").unwrap();
            for img in attachments.select(&img_selector) {
                if let Some(src) = img.value().attr("src") {
                    let image_url = if is_encrypted {
                        self.decrypt_media_url(src, "profile_images")?
                    } else {
                        self.normalize_media_url(src, "pic")?
                    };
                    pictures.push(image_url);
                }
            }

            // Extract videos
            let video_selector = Selector::parse("video:not(.gif)").unwrap();
            for video in attachments.select(&video_selector) {
                if let Some(url) = video.value().attr("data-url") {
                    let video_url = if is_encrypted {
                        self.decrypt_media_url(url, "video")?
                    } else {
                        self.normalize_media_url(url, "video")?
                    };
                    videos.push(video_url);
                }
            }

            // Extract GIFs
            let gif_selector = Selector::parse("video.gif").unwrap();
            for gif in attachments.select(&gif_selector) {
                if let Some(source) = gif.select(&Selector::parse("source").unwrap()).next() {
                    if let Some(src) = source.value().attr("src") {
                        let gif_url = if is_encrypted {
                            self.decrypt_media_url(src, "tweet_video_thumb")?
                        } else {
                            self.normalize_media_url(src, "tweet_video")?
                        };
                        gifs.push(gif_url);
                    }
                }
            }
        }

        Ok((pictures, videos, gifs))
    }

    fn decrypt_media_url(&self, encrypted_url: &str, media_type: &str) -> Result<String> {
        let encoded = encrypted_url.split("/enc/").nth(1)
            .context("Invalid encrypted URL format")?;
        let decoded = base64::decode(encoded)?;
        let url = String::from_utf8(decoded)?;
        
        Ok(format!("https://pbs.twimg.com/{}/{}", media_type, url))
    }

    fn normalize_media_url(&self, url: &str, media_type: &str) -> Result<String> {
        let normalized = url.split(&format!("/{}/", media_type)).nth(1)
            .context("Invalid media URL format")?;
        Ok(format!("https://pbs.twimg.com/{}/{}", media_type, normalized))
    }

    async fn parse_tweet(&self, element: scraper::ElementRef, is_encrypted: bool) -> Result<Tweet> {
        // Parse basic tweet information
        let text = self.extract_tweet_text(&element)?;
        let link = self.extract_tweet_link(&element)?;
        let date = self.extract_tweet_date(&element)?;
        let user = self.extract_user(&element, is_encrypted)?;
        let stats = self.extract_tweet_stats(&element)?;
        let external_link = self.extract_external_link(&element)?;
        let replying_to = self.extract_replying_to(&element)?;
        
        // Extract media content
        let (pictures, videos, gifs) = self.extract_media(element, is_encrypted).await?;

        // Check for quoted tweet
        let quoted_post = self.extract_quoted_tweet(&element, is_encrypted).await?;

        // Check tweet flags
        let is_retweet = element.select(&Selector::parse("div.retweet-header").unwrap()).next().is_some();
        let is_pinned = element.select(&Selector::parse("div.pinned").unwrap()).next().is_some();

        Ok(Tweet {
            link,
            text,
            user,
            date,
            is_retweet,
            is_pinned,
            external_link,
            replying_to,
            quoted_post,
            stats,
            pictures,
            videos,
            gifs,
        })
    }

    fn extract_tweet_text(&self, element: &scraper::ElementRef) -> Result<String> {
        let content_selector = Selector::parse("div.tweet-content.media-body").unwrap();
        let text = element
            .select(&content_selector)
            .next()
            .map(|el| el.text().collect::<String>())
            .unwrap_or_default()
            .trim()
            .to_string();
        Ok(text)
    }

    fn extract_tweet_link(&self, element: &scraper::ElementRef) -> Result<String> {
        let link = element
            .select(&Selector::parse("a.tweet-link").unwrap())
            .next()
            .and_then(|a| a.value().attr("href"))
            .map(|href| format!("https://twitter.com{}", href))
            .unwrap_or_default();
        Ok(link)
    }

    fn extract_tweet_date(&self, element: &scraper::ElementRef) -> Result<String> {
        let date = element
            .select(&Selector::parse("span.tweet-date a").unwrap())
            .next()
            .and_then(|a| a.value().attr("title"))
            .unwrap_or_default()
            .to_string();
        Ok(date)
    }

    fn extract_tweet_stats(&self, element: &scraper::ElementRef) -> Result<TweetStats> {
        let stat_selector = Selector::parse("span.tweet-stat").unwrap();
        let mut stats = element.select(&stat_selector);

        let parse_stat = |stat: Option<scraper::ElementRef>| -> i32 {
            stat.and_then(|s| s.text().next())
                .and_then(|t| t.replace(",", "").parse().ok())
                .unwrap_or(0)
        };

        Ok(TweetStats {
            comments: parse_stat(stats.next()),
            retweets: parse_stat(stats.nth(1)),
            quotes: parse_stat(stats.nth(2)),
            likes: parse_stat(stats.nth(3)),
        })
    }

    async fn extract_quoted_tweet(&self, element: &scraper::ElementRef, is_encrypted: bool) -> Result<Option<QuotedTweet>> {
        let quote_selector = Selector::parse("div.quote").unwrap();
        
        if let Some(quoted_element) = element.select(&quote_selector).next() {
            if quoted_element.value().attr("class").map_or(false, |c| c.contains("unavailable")) {
                return Ok(None);
            }

            let user = self.extract_user(&quoted_element, is_encrypted)?;
            let text = self.extract_tweet_text(&quoted_element)?;
            let link = self.extract_tweet_link(&quoted_element)?;
            let date = self.extract_tweet_date(&quoted_element)?;
            let (pictures, videos, gifs) = self.extract_media(quoted_element, is_encrypted).await?;

            Ok(Some(QuotedTweet {
                link,
                text,
                user,
                date,
                pictures,
                videos,
                gifs,
            }))
        } else {
            Ok(None)
        }
    }

    fn extract_user(&self, element: &scraper::ElementRef, is_encrypted: bool) -> Result<User> {
        let name = element
            .select(&Selector::parse("a.fullname").unwrap())
            .next()
            .map(|el| el.text().collect::<String>())
            .unwrap_or_default();

        let username = element
            .select(&Selector::parse("a.username").unwrap())
            .next()
            .map(|el| el.text().collect::<String>())
            .unwrap_or_default();

        let avatar_selector = Selector::parse("img.avatar").unwrap();
        let (avatar, profile_id) = if let Some(img) = element.select(&avatar_selector).next() {
            let src = img.value().attr("src").unwrap_or_default();
            if is_encrypted {
                let decoded = self.decrypt_media_url(src, "profile_images")?;
                let id = decoded.split("profile_images/").nth(1)
                    .and_then(|s| s.split("/").next())
                    .unwrap_or_default();
                (decoded, id.to_string())
            } else {
                let normalized = self.normalize_media_url(src, "profile_images")?;
                let id = normalized.split("profile_images/").nth(1)
                    .and_then(|s| s.split("/").next())
                    .unwrap_or_default();
                (normalized, id.to_string())
            }
        } else {
            (String::new(), String::new())
        };

        Ok(User {
            name,
            username,
            profile_id,
            avatar,
        })
    }

    fn extract_external_link(&self, element: &scraper::ElementRef) -> Result<String> {
        let link = element
            .select(&Selector::parse("a.card-container").unwrap())
            .next()
            .and_then(|a| a.value().attr("href"))
            .unwrap_or_default()
            .to_string();
        Ok(link)
    }

    fn extract_replying_to(&self, element: &scraper::ElementRef) -> Result<Vec<String>> {
        let mut replying_to = Vec::new();
        if let Some(reply_div) = element.select(&Selector::parse("div.replying-to").unwrap()).next() {
            for user in reply_div.select(&Selector::parse("a").unwrap()) {
                replying_to.push(user.text().collect::<String>());
            }
        }
        Ok(replying_to)
    }
}


// Implementation of additional utility traits
impl Default for Tweet {
    fn default() -> Self {
        Tweet {
            link: String::new(),
            text: String::new(),
            user: User::default(),
            date: String::new(),
            is_retweet: false,
            is_pinned: false,
            external_link: String::new(),
            replying_to: Vec::new(),
            quoted_post: None,
            stats: TweetStats::default(),
            pictures: Vec::new(),
            videos: Vec::new(),
            gifs: Vec::new(),
        }
    }
}

impl Default for User {
    fn default() -> Self {
        User {
            name: String::new(),
            username: String::new(),
            profile_id: String::new(),
            avatar: String::new(),
        }
    }
}

impl Default for TweetStats {
    fn default() -> Self {
        TweetStats {
            comments: 0,
            retweets: 0,
            quotes: 0,
            likes: 0,
        }
    }
}