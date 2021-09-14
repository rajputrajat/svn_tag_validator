use anyhow::Result;
use async_std::task;
use log::{info, trace};
use std::{collections::HashMap, env};
use svn_cmd::{Credentials, PathType, SvnCmd, SvnList};

#[async_std::main]
async fn main() -> Result<()> {
    env_logger::init();
    let args: Vec<_> = env::args().skip(1).collect();
    let path = args.get(0).expect("arg not given");
    trace!("check info of path: {:?}", &path);
    process_tag(path).await
}

async fn process_tag(path: &str) -> Result<()> {
    info!("Inspecting SVN path: {:#?}", path);
    let svn = SvnCmd::new(
        Credentials {
            username: "svc-p-blsrobo".to_owned(),
            password: "Comewel@12345".to_owned(),
        },
        None,
    )?;
    let list = svn.list(path, true).await?;
    let mut path_list: Vec<String> = Vec::new();
    let mut tasks = Vec::new();
    for e in list.filter(|e| e.kind == PathType::Dir) {
        let dir_path = format!("{}/{}", path, e.name);
        let cmd = format!("propget svn:externals {}", dir_path);
        let svn_clone = svn.clone();
        tasks.push(task::spawn(async move {
            svn_clone
                .raw_cmd(cmd)
                .await
                .unwrap_or_else(|_| "".to_owned())
        }));
    }
    task::block_on(async {
        for t in tasks {
            let out = t.await;
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
    });
    info!("paths: {:#?}", path_list);
    Ok(())
}

fn get_tags_map(svn_list: &SvnList, path: &str) -> HashMap<String, Vec<usize>> {
    let mut tag_indices_map: HashMap<String, Vec<usize>> = HashMap::new();
    svn_list
        .enumerate()
        .filter_map(|(i, e)| {
            if e.kind == PathType::Dir {
                Some((i, format!("{}/{}", path, e.name)))
            } else {
                None
            }
        })
        .filter(|(_i, p)| p.contains("tags"))
        .for_each(|(i, p)| {
            p.split('/').enumerate().for_each(|(j, s)| {
                if (s == "tags") && (j == (p.len() - 2)) {
                    tag_indices_map.insert(p.clone(), vec![i]);
                } else {
                    let key = tag_indices_map
                        .keys()
                        .find(|&k| p.contains(k))
                        .unwrap()
                        .clone();
                    if let Some(v) = tag_indices_map.get_mut(&key) {
                        v.push(i);
                    }
                }
            });
        });
    tag_indices_map
}
