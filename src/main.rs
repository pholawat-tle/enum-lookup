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

fn main() -> Result<()> {
    let config = AppConfig::new()?;

    let gitlab_client = Gitlab::new(config.gitlab_url, config.gitlab_token)
        .with_context(|| "Failed to create Gitlab client")?;

    for project in config.gitlab_projects {
        let tree_endpoint = Tree::builder()
            .project(project.name.clone())
            .ref_(project.branch.clone())
            .recursive(true)
            .build()?;

        let trees: Vec<RepoTreeObject> =
            api::paged(tree_endpoint, api::Pagination::All).query(&gitlab_client)?;

        let trees_iter = trees.into_iter();

        let enums_in_trees = trees_iter.flat_map(|tree| {
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

        println!("Total number of enums: {}", enums_in_trees.count());
    }

    Ok(())
}
