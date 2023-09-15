use anyhow::{Context, Result};
use enum_lookup::{
    scala::{CaseObjectParser, EnumerationParser},
    AppConfig, Enum, Parser,
};
use gitlab::{
    api::{
        self,
        projects::repository::{files::FileRaw, Tree},
        Query,
    },
    Gitlab, RepoTreeObject,
};
use rayon::prelude::{IntoParallelIterator, ParallelIterator};

fn main() -> Result<()> {
    let config = AppConfig::new()?;

    let gitlab_client = Gitlab::new(config.gitlab_url, config.gitlab_token)
        .with_context(|| "Failed to create Gitlab client")?;

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(100)
        .build()
        .unwrap();

    let all_enums: Vec<Enum> = pool.install(|| {
        config
            .gitlab_projects
            .into_par_iter()
            .flat_map(|project| {
                let tree_endpoint = Tree::builder()
                    .project(project.name.clone())
                    .ref_(project.branch.clone())
                    .recursive(true)
                    .build();

                let tree_endpoint = match tree_endpoint {
                    Ok(tree_endpoint) => tree_endpoint,
                    Err(_e) => return vec![],
                };

                let git_trees =
                    api::paged(tree_endpoint, api::Pagination::All).query(&gitlab_client);

                let git_trees: Vec<RepoTreeObject> = match git_trees {
                    Ok(trees) => trees,
                    Err(_e) => return vec![],
                };

                let enum_in_current_repo = git_trees
                    .into_par_iter()
                    .flat_map(|git_tree| {
                        let mut enums_in_current_file: Vec<Enum> = vec![];

                        if git_tree.mode != "100644" {
                            return enums_in_current_file;
                        };

                        let file_extension = git_tree.name.split(".").last();
                        let file_endpoint = FileRaw::builder()
                            .project(project.name.clone())
                            .ref_(project.branch.clone())
                            .file_path(git_tree.path)
                            .build();

                        let file_endpoint = match file_endpoint {
                            Ok(file_endpoint) => file_endpoint,
                            Err(_e) => return enums_in_current_file,
                        };

                        let file_content = api::raw(file_endpoint).query(&gitlab_client);

                        let file_content = match file_content {
                            Ok(file_content) => file_content,
                            Err(_e) => return enums_in_current_file,
                        };

                        let file_content = match String::from_utf8(file_content) {
                            Ok(file_content) => file_content,
                            Err(_e) => return enums_in_current_file,
                        };

                        match file_extension {
                            Some("scala") => {
                                println!("Executing Scala Parsers: {}", git_tree.name);
                                enums_in_current_file
                                    .extend(EnumerationParser::parse_enums(&file_content));
                                enums_in_current_file
                                    .extend(CaseObjectParser::parse_enums(&file_content));
                            }
                            _ => {
                                println!("No parser for file: {}", git_tree.name);
                            }
                        }

                        enums_in_current_file
                    })
                    .collect();

                enum_in_current_repo
            })
            .collect()
    });

    println!("Enums: {:#?}", all_enums);

    Ok(())
}
