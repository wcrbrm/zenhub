use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::error::Error;
use structopt::StructOpt;

#[derive(StructOpt, Debug, Clone)]
#[structopt(name = "basic")]
struct Opt {
    /// zen hub api root
    #[structopt(
        long,
        env = "ZENHUB_API_ROOT",
        default_value = "https://api.zenhub.com"
    )]
    api_root: String,

    /// zen hub workspace ID
    #[structopt(long, env = "ZENHUB_WORKSPACE_ID")]
    workspace_id: String,

    /// zen hub api
    #[structopt(long, env = "ZENHUB_API_TOKEN")]
    api_token: String,

    /// zen agent
    #[structopt(long, env = "ZENHUB_AGENT", default_value = "webapp/2.45.17")]
    agent: String,
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

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ZenhubRepository {
    /// Github repository ID
    gh_id: u64,
    /// Github repository name
    name: String,
    /// Owner of the repository
    owner_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ZenhubRepositoriesResponseDataWorkspace {
    id: String,
    name: String,
    description: String,
    repositories: Vec<ZenhubRepository>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ZenhubRepositoriesResponseData {
    workspace: ZenhubRepositoriesResponseDataWorkspace,
}

#[derive(Serialize, Deserialize, Debug)]
struct ZenhubRepositoriesResponse {
    data: ZenhubRepositoriesResponseData,
}

#[derive(Serialize, Deserialize, Debug)]
struct ZenhubIssue {
    issue_number: u64,
    repo_id: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct ZenhubPipeline {
    name: String,
    description: Option<String>,
    _id: String,
    issues: Vec<ZenhubIssue>,
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

#[allow(dead_code)]
async fn read_issues(
    opt: Opt,
    repositories: Vec<ZenhubRepository>,
) -> Result<String, Box<dyn Error>> {
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

    println!("{}", url);
    let res = reqwest::Client::new()
        .get(&url)
        .headers(zenhub_headers(opt))
        .send()
        .await?
        .text()
        .await?;
    println!("{}", res);
    Ok(res)
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let opt = Opt::from_args();
    //    println!("Options {:#?}", opt);

    //    let resp_user = read_user(opt.clone()).await.unwrap();
    //    println!("User\t{:#?}", resp_user.github.email);

    let repositories = read_repositories(opt.clone()).await.unwrap();
    read_issues(opt, repositories).await?;

    //    for repo in repositories {
    //         println!("{}\t{}", repo.gh_id, repo.name);
    //    }

    //  let board = read_pipelines(opt.clone()).await.unwrap();
    //  let pipelines = board.pipelines.iter().map(|x| &x.name).collect::<HashSet<_>>();
    //  println!("Pipelines\t{:#?}", pipelines);

    Ok(())
}
