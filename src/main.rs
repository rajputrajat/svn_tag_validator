use anyhow::Result;
use async_std::task;
use log::trace;
use std::{collections::HashMap, env, time::Instant};
use svn_cmd::{Credentials, PathType, SvnCmd, SvnList};

#[async_std::main]
async fn main() -> Result<()> {
    env_logger::init();
    let start = Instant::now();
    let args: Vec<_> = env::args().skip(1).collect();
    let path = args.get(0).expect("arg not given");
    trace!("check info of path: {:?}", &path);
    process_tag(path, &start).await
}

async fn process_tag(path: &str, start_instance: &Instant) -> Result<()> {
    println!("Inspecting SVN path: {:#?}", path);
    let svn = SvnCmd::new(
        Credentials {
            username: "svc-p-blsrobo".to_owned(),
            password: "Comewel@12345".to_owned(),
        },
        None,
    )?;
    let path = remove_last_slash(path);
    let list = svn.list(&path, true).await?;
    println!(
        "SvnList data received in '{}' msec.",
        start_instance.elapsed().as_millis()
    );
    trace!("{:?}", list);
    let mut path_list: Vec<(String, Vec<String>)> = Vec::new();
    let mut tasks = Vec::new();

    let tag_indices_map = { get_tags_map(&list, &path) };
    trace!("{:?}", tag_indices_map);
    for (k, v) in tag_indices_map.into_iter() {
        let author = {
            let le = &list
                .iter()
                .find(|e| k == format!("{}/{}", &path, e.name))
                .unwrap();
            le.commit.author.to_owned()
        };
        for entry in v.iter().map(|&i| list.iter().nth(i).unwrap()) {
            let k = k.clone();
            let author = author.clone();
            let dir_path = format!("{}/{}", path, entry.name);
            let cmd = format!("propget svn:externals {}", dir_path);
            let svn_clone = svn.clone();
            tasks.push(task::spawn(async move {
                (
                    k,
                    author,
                    svn_clone
                        .raw_cmd(cmd)
                        .await
                        .unwrap_or_else(|_| "".to_owned()),
                )
            }));
        }
    }

    task::block_on(async {
        for t in tasks {
            let (key, author, out) = t.await;
            let new_non_tags = out
                .split(&['\n', '\r'][..])
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
                println!(
                    "Non tags externals for '{}', {}: {:#?}",
                    key, author, new_non_tags
                );
            }
            path_list.push((key, new_non_tags));
        }
    });
    println!(
        "Completed in '{}' msec.",
        start_instance.elapsed().as_millis()
    );
    Ok(())
}

fn get_tags_map(svn_list: &SvnList, path: &str) -> HashMap<String, Vec<usize>> {
    let mut tag_indices_map: HashMap<String, Vec<usize>> = HashMap::new();
    svn_list
        .iter()
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
            let path_split: Vec<&str> = p.split('/').collect();
            path_split.iter().enumerate().for_each(|(j, &s)| {
                if (s == "tags") && (j == (path_split.len() - 2)) {
                    tag_indices_map.insert(p.clone(), vec![]);
                }
            });
            let keys = tag_indices_map
                .keys()
                .map(|s| s.to_owned())
                .collect::<Vec<_>>();
            for key in keys {
                trace!("check if svn-path '{}' can be placed in '{}'", p, &key);
                if p.contains(&key) {
                    if let Some(v) = tag_indices_map.get_mut(&key) {
                        v.push(i);
                    }
                }
            }
        });
    tag_indices_map
}

fn remove_last_slash(input_str: &str) -> String {
    input_str.strip_suffix('/').unwrap_or(input_str).to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_last_slash_removed() {
        assert_eq!(remove_last_slash("hello/there/"), "hello/there".to_owned());
    }

    #[test]
    fn check_last_slash_removed_not() {
        assert_eq!(remove_last_slash("hello/there"), "hello/there".to_owned());
    }
}
