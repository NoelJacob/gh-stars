use anyhow::{Context, Result};
use octocrab::models::Repository;
use std::sync::Arc;

use crate::ai::AiSuggester;
use crate::config::Config;
use crate::github::{GitHubClient, GithubList};

pub enum AppState {
    Loading,
    RepoList,
    ListSelect,
    Confirmation,
    Error(String),
}

pub struct App {
    pub state: AppState,
    pub config: Config,
    pub github_client: Arc<GitHubClient>,
    pub ai_suggester: Option<AiSuggester>,
    pub starred_repos: Vec<Repository>,
    pub user_lists: Vec<GithubList>,
    pub selected_repo_idx: usize,
    pub selected_list_idx: usize,
    pub suggested_list: Option<String>,
    pub scroll_offset: usize,
    pub message: Option<String>,
    pub should_quit: bool,
}

impl App {
    pub async fn new(config: Config) -> Result<Self> {
        let github_client = Arc::new(
            GitHubClient::new(
                config.github_token.clone().unwrap(),
                config.github_username.clone().unwrap(),
            )
            .await
            .context("Failed to initialize GitHub client")?,
        );

        let ai_suggester = config.openai_api_key.clone().map(AiSuggester::new);

        Ok(Self {
            state: AppState::Loading,
            config,
            github_client,
            ai_suggester,
            starred_repos: Vec::new(),
            user_lists: Vec::new(),
            selected_repo_idx: 0,
            selected_list_idx: 0,
            suggested_list: None,
            scroll_offset: 0,
            message: None,
            should_quit: false,
        })
    }

    pub async fn load_data(&mut self) -> Result<()> {
        self.state = AppState::Loading;
        
        // Load starred repositories
        self.starred_repos = self.github_client
            .get_starred_repos()
            .await
            .context("Failed to load starred repositories")?;
            
        // Load user lists
        self.user_lists = self.github_client
            .get_user_lists()
            .await
            .context("Failed to load user lists")?;
            
        // If no repositories or lists, show an error
        if self.starred_repos.is_empty() {
            self.state = AppState::Error("No starred repositories found.".to_string());
            return Ok(());
        }
        
        if self.user_lists.is_empty() {
            self.state = AppState::Error("No GitHub lists found. Please create at least one list.".to_string());
            return Ok(());
        }
        
        self.state = AppState::RepoList;
        Ok(())
    }

    pub fn next_repo(&mut self) {
        if !self.starred_repos.is_empty() {
            self.selected_repo_idx = (self.selected_repo_idx + 1) % self.starred_repos.len();
            self.suggested_list = None; // Reset suggestion when changing repos
        }
    }

    pub fn prev_repo(&mut self) {
        if !self.starred_repos.is_empty() {
            self.selected_repo_idx = if self.selected_repo_idx > 0 {
                self.selected_repo_idx - 1
            } else {
                self.starred_repos.len() - 1
            };
            self.suggested_list = None; // Reset suggestion when changing repos
        }
    }

    pub fn next_list(&mut self) {
        if !self.user_lists.is_empty() {
            self.selected_list_idx = (self.selected_list_idx + 1) % self.user_lists.len();
        }
    }

    pub fn prev_list(&mut self) {
        if !self.user_lists.is_empty() {
            self.selected_list_idx = if self.selected_list_idx > 0 {
                self.selected_list_idx - 1
            } else {
                self.user_lists.len() - 1
            };
        }
    }

    pub async fn get_ai_suggestion(&mut self) -> Result<()> {
        if let Some(ai) = &self.ai_suggester {
            if let Some(repo) = self.starred_repos.get(self.selected_repo_idx) {
                self.message = Some("Generating AI suggestion...".to_string());
                match ai.suggest_list(repo, &self.user_lists).await {
                    Ok(suggestion) => {
                        self.suggested_list = suggestion;
                        
                        // If we got a suggestion, try to select that list
                        if let Some(list_name) = &self.suggested_list {
                            if let Some(index) = self.user_lists.iter().position(|l| &l.name == list_name) {
                                self.selected_list_idx = index;
                            }
                        }
                        
                        self.message = None;
                    },
                    Err(e) => {
                        self.message = Some(format!("AI suggestion error: {}", e));
                    }
                }
            }
        } else {
            self.message = Some("OpenAI API key not configured.".to_string());
        }
        
        Ok(())
    }

    pub async fn add_current_repo_to_list(&mut self) -> Result<()> {
        if let (Some(repo), Some(list)) = (
            self.starred_repos.get(self.selected_repo_idx), 
            self.user_lists.get(self.selected_list_idx)
        ) {
            let owner = repo.owner.as_ref()
                .map(|o| o.login.clone())
                .unwrap_or_default();
                
            self.github_client
                .add_repo_to_list(&owner, &repo.name, &list.name)
                .await?;
                
            self.message = Some(format!(
                "Added '{}' to list '{}'", 
                repo.name, 
                list.name
            ));
        }
        
        Ok(())
    }
    
    pub fn select_repo_list(&mut self) {
        self.state = AppState::RepoList;
    }
    
    pub fn select_list_selector(&mut self) {
        self.state = AppState::ListSelect;
    }
    
    pub fn confirm_action(&mut self) {
        self.state = AppState::Confirmation;
    }
    
    pub fn set_error(&mut self, error: String) {
        self.state = AppState::Error(error);
    }
}