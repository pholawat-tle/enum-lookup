use anyhow::{Context, Result};
use config::{Config, Environment};

pub struct AppConfig {
    pub gitlab_url: String,
    pub gitlab_token: String,
    pub gitlab_projects: Vec<Project>,
}

pub struct Project {
    pub name: String,
    pub branch: String,
}

impl AppConfig {
    pub fn new() -> Result<AppConfig> {
        let builder = Config::builder();
        let env_config = builder
            .add_source(Environment::with_prefix("APP"))
            .build()
            .with_context(|| "Failed to build config")?;

        let gitlab_token = env_config
            .get_string("gitlab_token")
            .with_context(|| "Failed to get gitlab_token from config")?;

        let gitlab_url = env_config
            .get_string("gitlab_url")
            .with_context(|| "Failed to get gitlab_url from config")?;

        let project_settings = env_config
            .get_string("gitlab_projects")
            .with_context(|| "Failed to get gitlab_projects from config")?;

        let gitlab_projects = project_settings
            .split(",")
            .collect::<Vec<&str>>()
            .chunks_exact(2)
            .map(|chunk| Project {
                name: chunk[0].to_string(),
                branch: chunk[1].to_string(),
            })
            .collect();

        Ok(AppConfig {
            gitlab_url,
            gitlab_token,
            gitlab_projects,
        })
    }
}
