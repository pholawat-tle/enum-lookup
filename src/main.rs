use anyhow::{Context, Result};
use enum_lookup::AppConfig;
use gitlab::{
    api::{self, projects::repository::Tree, Query},
    Gitlab, RepoTreeObject,
};

fn main() -> Result<()> {
    let config = AppConfig::new()?;

    let gitlab_client = Gitlab::new(config.gitlab_url, config.gitlab_token)
        .with_context(|| "Failed to create Gitlab client")?;

    for project in config.gitlab_projects {
        let tree_endpoint = Tree::builder()
            .project(project.name)
            .ref_(project.branch)
            .recursive(true)
            .build()?;

        let trees: Vec<RepoTreeObject> =
            api::paged(tree_endpoint, api::Pagination::All).query(&gitlab_client)?;

        for tree in trees {
            if tree.mode == "100644" {
                let file_extension = tree.name.split(".").last();
                match file_extension {
                    Some("scala") => {
                        println!("Executing Scala Parsers: {}", tree.name)
                    }
                    _ => (),
                }
            }
        }
    }

    Ok(())
}
