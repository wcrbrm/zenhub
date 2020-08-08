use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::error::Error;
use structopt::StructOpt;

#[derive(StructOpt, Debug, Clone)]
#[structopt(
    name = "zenhub",
    about = "A command line quickie to view zenhub pipelines board"
)]
struct Opt {
    /// zen hub api root
    #[structopt(
        long,
        env = "ZENHUB_API_ROOT",
        default_value = "https://api.zenhub.com",
        hidden = true
    )]
    api_root: String,
    /// zen hub workspace ID
    #[structopt(long, env = "ZENHUB_WORKSPACE_ID")]
    workspace_id: String,

    /// zen hub api
    #[structopt(long, env = "ZENHUB_API_TOKEN", hide_env_values = true)]
    api_token: String,

    /// zen agent
    #[structopt(long, env = "ZENHUB_AGENT", default_value = "webapp/2.45.17")]
    agent: String,

    /// pipelines to be rendered
    #[structopt(long, short)]
    pipeline: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ZenhubGithubUser {
    id: u64,
    username: String,
    name: String,
    avatar_url: String,
    email: String,
    followers: Option<u64>,
    following: Option<u64>,
    public_repos: Option<u64>,
    created_at: Option<String>,
    company: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ZenhubUserResponse {
    id: String,
    github: ZenhubGithubUser,
    created_at: Option<String>, // DateTime
    last_auth: Option<String>,  // DateTime
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct ZenhubRepository {
    /// Github repository ID
    gh_id: u64,
    /// Github repository name
    name: String,
    /// Owner of the repository
    owner_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ZenhubRepositoriesResponseDataWorkspace {
    id: String,
    name: String,
    description: String,
    repositories: Vec<ZenhubRepository>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ZenhubRepositoriesResponseData {
    workspace: ZenhubRepositoriesResponseDataWorkspace,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ZenhubRepositoriesResponse {
    data: ZenhubRepositoriesResponseData,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ZenhubIssue {
    issue_number: u64,
    repo_id: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ZenhubAssignee {
    html_url: Option<String>,
    avatar_url: Option<String>,
    login: String,
    id: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ZenhubLabel {
    color: Option<String>,
    name: String,
    id: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ZenhubMilestone {
    state: String,
    number: u64,
    title: String,
    due_on: Option<String>,
    id: u64,
    updated_at: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ZenhubPipeline {
    name: String,
    description: Option<String>,
    _id: String,
    issues: Option<Vec<ZenhubIssue>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ZenhubIssueInfo {
    assignee: Option<ZenhubAssignee>,
    assignees: Vec<ZenhubAssignee>,
    created_at: String,
    closed_at: Option<String>,
    estimate: Option<f32>,
    html_url: String,
    is_epic: bool,
    labels: Vec<ZenhubLabel>,
    milestone: Option<ZenhubMilestone>,
    number: Option<u32>,
    repo_name: String,
    organization_name: Option<String>,
    parent_epics: Vec<ZenhubIssue>,
    state: String,
    title: String,
    updated_at: Option<String>,
    user: Option<ZenhubAssignee>,
    issue_number: u64,
    pipeline: Option<ZenhubPipeline>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ZenhubBoardResponse {
    _id: String,
    name: String,
    pipelines: Vec<ZenhubPipeline>,
}

#[allow(dead_code)]
fn zenhub_headers(opt: Opt) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("X-Authentication-Token", opt.api_token.parse().unwrap());
    headers.insert("X-Zenhub-Agent", opt.agent.parse().unwrap());
    headers.insert("Content-Type", "application/json".parse().unwrap());
    headers
}

#[allow(dead_code)]
async fn read_user(opt: Opt) -> Result<ZenhubUserResponse, Box<dyn Error>> {
    let url: String = format!("{}/v1/user", opt.api_root);
    let response: ZenhubUserResponse = reqwest::Client::new()
        .get(&url)
        .headers(zenhub_headers(opt))
        .send()
        .await?
        .json()
        .await?;
    Ok(response)
}

#[allow(dead_code)]
async fn read_pipelines(opt: Opt) -> Result<ZenhubBoardResponse, Box<dyn Error>> {
    let url: String = format!("{}/v5/workspaces/{}/board", opt.api_root, opt.workspace_id);
    let res = reqwest::Client::new()
        .get(&url)
        .headers(zenhub_headers(opt))
        .send()
        .await?
        .json()
        .await?;
    Ok(res)
}

#[derive(Serialize, Deserialize, Clone)]
struct ZenhubIssuesFilter {
    by_assignee: Option<String>,
    by_pipeline_name: Option<String>,
}

#[allow(dead_code)]
async fn read_issues(
    opt: Opt,
    repositories: Vec<ZenhubRepository>,
    filter: &ZenhubIssuesFilter,
) -> Result<Vec<ZenhubIssueInfo>, Box<dyn Error>> {
    let ids = repositories
        .iter()
        .map(|x| format!("{}", x.gh_id))
        .collect::<HashSet<_>>();
    let ids_str: String = ids.iter().map(|x| &**x).collect::<Vec<_>>().join(",");

    let mut url: String = format!(
        "{}/v5/workspaces/{}/issues?repo_ids={}",
        opt.api_root, opt.workspace_id, ids_str
    );

    url.push_str("&epics=1");
    url.push_str("&estimates=1");
    url.push_str("&connections=1");
    url.push_str("&forceUpdate=0");
    url.push_str("&pipelines=1");
    url.push_str("&priorities=1");
    url.push_str("&releases=1");

    let res = reqwest::Client::new()
        .get(&url)
        .headers(zenhub_headers(opt))
        .send()
        .await?
        .json::<Vec<ZenhubIssueInfo>>()
        .await?;
    let filtered = res
        .clone()
        .drain(..)
        .filter(|x| {
            let mut m = true;
            if let Some(by_assignee) = filter.by_assignee.to_owned().take() {
                if let Some(assignee) = x.clone().assignee.take() {
                    m = m && (assignee.login == by_assignee);
                } else {
                    m = false
                }
            }
            if let Some(by_pipeline_name) = filter.by_pipeline_name.to_owned().take() {
                if let Some(pipeline) = x.clone().pipeline.take() {
                    m = m && (pipeline.name == by_pipeline_name)
                } else {
                    m = false
                }
            }
            m
        })
        .collect::<Vec<ZenhubIssueInfo>>();
    Ok(filtered)
}

#[allow(dead_code)]
async fn read_repositories(opt: Opt) -> Result<Vec<ZenhubRepository>, Box<dyn Error>> {
    let url: String = format!("{}/v1/graphql", opt.api_root);
    let payload = format!(
        r###"{{"query":"{{
        workspace(id: \"{}\") {{
            ...space
        }}
    }}
    fragment space on Workspace {{
        id
        name
        description
        repositories {{
            ghId
            name
            ownerName
        }}
    }}
"}}"###,
        opt.workspace_id
    )
    .replace('\n', "\\n");
    // println!("url={}\n{}\n", url, payload);

    let r = reqwest::Client::new()
        .post(&url)
        .headers(zenhub_headers(opt))
        .body(payload)
        .send()
        .await?
        .json::<ZenhubRepositoriesResponse>()
        .await?;
    // println!("{:#?}", r.data.workspace.repositories);
    Ok(r.data.workspace.repositories)
}

fn display_issues(list: Vec<ZenhubIssueInfo>) {
    for i in list {
        let mut estimate_str: String = "".to_string();
        if let Some(est) = i.clone().estimate.take() {
            estimate_str = format!("{}", est);
        }
        println!(
            "{}#{}\t{}h\t{}",
            i.repo_name,
            i.issue_number,
            estimate_str,
            i.title.trim(),
        )
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let opt = Opt::from_args();
    //    println!("Options {:#?}", opt);

    let resp_user = read_user(opt.clone()).await.unwrap();
    println!(
        "User\t{}\t{}",
        resp_user.github.username, resp_user.github.email
    );

    let repositories = read_repositories(opt.clone()).await.unwrap();
    let username = Some(resp_user.github.username);
    println!("# Backlog");
    display_issues(
        read_issues(
            opt.clone(),
            repositories.clone(),
            &ZenhubIssuesFilter {
                by_assignee: username.clone(),
                by_pipeline_name: Some("Backlog".to_string()),
            },
        )
        .await?,
    );

    println!("# Second Backlog");
    display_issues(
        read_issues(
            opt.clone(),
            repositories.clone(),
            &ZenhubIssuesFilter {
                by_assignee: username.clone(),
                by_pipeline_name: Some("Backlog".to_string()),
            },
        )
        .await?,
    );

    //    for repo in repositories {
    //         println!("{}\t{}", repo.gh_id, repo.name);
    //    }

    // let board = read_pipelines(opt.clone()).await.unwrap();
    // let pipelines = board
    // .pipelines
    // .iter()
    // .map(|x| &x.name)
    // .collect::<HashSet<_>>();
    // println!("Pipelines\t{:#?}", pipelines);

    Ok(())
}
