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

    let enums: Vec<Enum> = pool.install(|| {
        let projects_iter = config.gitlab_projects.into_par_iter();
        let enums = projects_iter.flat_map(|project| -> Vec<Enum> {
            let tree_endpoint = Tree::builder()
                .project(project.name.clone())
                .ref_(project.branch.clone())
                .recursive(true)
                .build();

            let tree_endpoint = match tree_endpoint {
                Ok(tree_endpoint) => tree_endpoint,
                Err(_e) => return vec![],
            };

            let trees = api::paged(tree_endpoint, api::Pagination::All).query(&gitlab_client);

            let trees: Vec<RepoTreeObject> = match trees {
                Ok(trees) => trees,
                Err(_e) => return vec![],
            };

            let trees_iter = trees.into_par_iter();
            let enums_in_project = trees_iter.flat_map(|tree| {
                let mut enums_in_file: Vec<Enum> = vec![];

                if tree.mode != "100644" {
                    return enums_in_file;
                };

                let file_extension = tree.name.split(".").last();
                let file_endpoint = FileRaw::builder()
                    .project(project.name.clone())
                    .ref_(project.branch.clone())
                    .file_path(tree.path)
                    .build();

                let file_endpoint = match file_endpoint {
                    Ok(file_endpoint) => file_endpoint,
                    Err(_e) => return enums_in_file,
                };

                let file_content = api::raw(file_endpoint).query(&gitlab_client);

                let file_content = match file_content {
                    Ok(file_content) => file_content,
                    Err(_e) => return enums_in_file,
                };

                let file_content = match String::from_utf8(file_content) {
                    Ok(file_content) => file_content,
                    Err(_e) => return enums_in_file,
                };

                match file_extension {
                    Some("scala") => {
                        println!("Executing Scala Parsers: {}", tree.name);
                        enums_in_file.extend(EnumerationParser::parse_enums(&file_content));
                        enums_in_file.extend(CaseObjectParser::parse_enums(&file_content));
                    }
                    _ => (),
                }

                enums_in_file
            });

            enums_in_project.collect()
        });

        enums.collect()
    });

    println!("Enums: {:#?}", enums);

    Ok(())
}
