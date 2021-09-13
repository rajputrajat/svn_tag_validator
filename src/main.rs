use anyhow::Result;
use log::{info, trace};
use std::env;
use svn_cmd::{Credentials, PathType, SvnCmd};

#[async_std::main]
async fn main() -> Result<()> {
    env_logger::init();
    let args: Vec<_> = env::args().skip(1).collect();
    let path = args.get(0).expect("arg not given");
    trace!("check info of path: {:?}", &path);
    process_tag(path).await
}

async fn process_tag(path: &str) -> Result<()> {
    let svn = SvnCmd::new(
        Credentials {
            username: "svc-p-blsrobo".to_owned(),
            password: "Comewel@12345".to_owned(),
        },
        None,
    )?;
    let list = svn.list(path, true).await?;
    let mut path_list: Vec<String> = Vec::new();
    for e in list.filter(|e| e.kind == PathType::Dir) {
        let _ = async {
            let dir_path = format!("{}/{}", path, e.name);
            let out = svn
                .raw_cmd(&format!("propget svn:externals {}", dir_path))
                .await
                .unwrap_or_else(|_| "".to_owned());
            let new_non_tags = out
                .split_whitespace()
                .filter(|&s| !s.is_empty())
                .filter_map(|s| {
                    if s.contains("tags") {
                        None
                    } else {
                        Some(s.to_owned())
                    }
                })
                .collect::<Vec<_>>();
            if !new_non_tags.is_empty() {
                info!("Non tags external items: {:#?}", new_non_tags);
            }
            path_list.extend_from_slice(&new_non_tags);
        }
        .await;
    }
    info!("paths: {:#?}", path_list);
    Ok(())
}
