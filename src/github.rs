use anyhow::{Context, Result};
use octocrab::{Octocrab, models::Repository};
use std::sync::Arc;

pub struct GitHubClient {
    client: Arc<Octocrab>,
    username: String,
}

impl GitHubClient {
    pub async fn new(token: String, username: String) -> Result<Self> {
        let client = Octocrab::builder()
            .personal_token(token)
            .build()
            .context("Failed to create GitHub client")?;
            
        Ok(Self {
            client: Arc::new(client),
            username,
        })
    }
    
    pub async fn get_starred_repos(&self) -> Result<Vec<Repository>> {
        let page: serde_json::Value = self.client
            .get("user/starred", None::<&()>)
            .await
            .context("Failed to get starred repos")?;
        
        let mut all_repos: Vec<Repository> = Vec::new();
        
        // Parse page content
        if let Some(items) = page.get("items").and_then(|i| i.as_array()) {
            for item in items {
                if let Ok(repo) = serde_json::from_value::<Repository>(item.clone()) {
                    all_repos.push(repo);
                }
            }
        }
        
        // Handle pagination if needed
        // This is simplified - in a real app, you'd need to follow GitHub's pagination links
        
        Ok(all_repos)
    }
    
    pub async fn get_user_lists(&self) -> Result<Vec<GithubList>> {
        // Note: GitHub's API for starred lists is not public yet, so this is a placeholder
        // You'll need to adapt this when the API becomes available
        
        // For now, we'll create some mock lists for testing purposes
        let lists = vec![
            GithubList {
                name: "Frontend".to_string(),
                description: Some("Frontend development projects and libraries".to_string()),
            },
            GithubList {
                name: "Backend".to_string(),
                description: Some("Backend development and server technologies".to_string()),
            },
            GithubList {
                name: "Tools".to_string(),
                description: Some("Development tools and utilities".to_string()),
            },
            GithubList {
                name: "Learning".to_string(),
                description: Some("Educational resources and learning projects".to_string()),
            },
        ];
        
        Ok(lists)
    }
    
    pub async fn add_repo_to_list(&self, repo_owner: &str, repo_name: &str, list_name: &str) -> Result<()> {
        // This is a placeholder since GitHub's API for adding repos to lists is not public
        // In a real implementation, you would call the API endpoint here
        
        log::info!(
            "Adding repository {}/{} to list '{}'", 
            repo_owner, 
            repo_name, 
            list_name
        );
        
        // Simulate success
        Ok(())
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GithubList {
    pub name: String,
    pub description: Option<String>,
}