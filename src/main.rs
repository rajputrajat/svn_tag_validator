use anyhow::Result;
use async_std::path::Path;
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
    list.filter(|e| e.kind == PathType::Dir).for_each(|e| {
        let _ = async {
            let dir_path = Path::new(path).join(e.name);
            let dir_path = dir_path.to_str().unwrap();
            let out = svn
                .raw_cmd(&format!("propget svn:externals {}", dir_path))
                .await
                .unwrap_or_else(|_| "".to_owned());
            path_list.extend_from_slice(
                &out.split_whitespace()
                    .filter(|&s| !s.is_empty())
                    .filter_map(|s| {
                        if s.contains("tags") {
                            None
                        } else {
                            Some(s.to_owned())
                        }
                    })
                    .collect::<Vec<_>>(),
            );
        };
    });
    info!("paths: {:#?}", path_list);
    Ok(())
}
