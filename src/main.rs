// use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    /// zen hub api root
    #[structopt(long, env="ZENHUB_API_ROOT", default_value="https://api.zenhub.com")]
    api_root: String,

    /// zen hub workspace ID
    #[structopt(long, env="ZENHUB_WORKSPACE_ID")]
    workspace_id: String,
    
    /// zen hub api
    #[structopt(long, env="ZENHUB_API_TOKEN")]
    api_token: String,
}


fn main() {
    let opt = Opt::from_args();  
    println!("Hello, {:#?}", opt);
}
