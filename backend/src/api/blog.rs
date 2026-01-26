use axum::{extract::State, http::StatusCode, response::Json};
use serde::{Deserialize, Serialize};
use crate::api::state::AppState;

#[derive(Serialize, Deserialize)]
pub struct BlogPost {
    pub id: String,
    pub title: String,
    pub excerpt: String,
    pub content: String,
    pub author: String,
    pub published_date: String,
    pub read_time: u32,
    pub tags: Vec<String>,
    pub featured_image: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct BlogResponse {
    pub posts: Vec<BlogPost>,
    pub total: u32,
    pub page: u32,
    pub per_page: u32,
}

pub async fn get_blog_posts(
    State(_state): State<AppState>,
) -> Result<Json<BlogResponse>, StatusCode> {
    // Mock blog posts - in production, this would come from a database
    let posts = vec![
        BlogPost {
            id: "1".to_string(),
            title: "The Future of Crypto Payment Security".to_string(),
            excerpt: "Exploring the latest security innovations in cryptocurrency payment processing and how FidduPay stays ahead of threats.".to_string(),
            content: "Full article content would be here...".to_string(),
            author: "FidduPay Security Team".to_string(),
            published_date: "2026-01-25T00:00:00Z".to_string(),
            read_time: 5,
            tags: vec!["security".to_string(), "crypto".to_string(), "payments".to_string()],
            featured_image: None,
        },
        BlogPost {
            id: "2".to_string(),
            title: "Multi-Chain Payments: The New Standard".to_string(),
            excerpt: "Why businesses are adopting multi-blockchain payment solutions and how to choose the right networks for your needs.".to_string(),
            content: "Full article content would be here...".to_string(),
            author: "FidduPay Engineering Team".to_string(),
            published_date: "2026-01-20T00:00:00Z".to_string(),
            read_time: 7,
            tags: vec!["blockchain".to_string(), "multi-chain".to_string(), "business".to_string()],
            featured_image: None,
        },
        BlogPost {
            id: "3".to_string(),
            title: "Integrating FidduPay: A Developer's Guide".to_string(),
            excerpt: "Step-by-step tutorial on integrating cryptocurrency payments into your application using our Node.js SDK.".to_string(),
            content: "Full article content would be here...".to_string(),
            author: "FidduPay Developer Relations".to_string(),
            published_date: "2026-01-15T00:00:00Z".to_string(),
            read_time: 10,
            tags: vec!["development".to_string(), "sdk".to_string(), "tutorial".to_string()],
            featured_image: None,
        },
    ];

    let response = BlogResponse {
        total: posts.len() as u32,
        page: 1,
        per_page: 10,
        posts,
    };

    Ok(Json(response))
}
