use anyhow::Result;
use log::{info, trace};
use std::env;
use svn_cmd::{Credentials, SvnCmd};

trait Rule {}

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
    let info = svn.info(path).await?;
    info!("SvnInfo: {:?}", info);
    Ok(())
}
