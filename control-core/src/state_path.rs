use std::{
    env, fs,
    io::{Error, Result},
};

pub const ERRMSG: &str = "Couldn't determine data directory. This Application needs either permissions to create /var/lib/qitech or a home directory.";

pub fn get() -> Result<String> {
    let sys_path = env::var("STATE_DIRECTORY").unwrap_or("/var/lib".to_owned());
    let qitech_path = sys_path + "/qitech";
    let qitech_exists = fs::exists(&qitech_path).unwrap_or(false)
        && match fs::metadata(&qitech_path) {
            Ok(dir) => dir.permissions().readonly(),
            Err(_) => false,
        };
    if !qitech_exists && fs::create_dir_all(&qitech_path).is_err() {
        let home_path = determine_user_state_home()?;
        fs::create_dir_all(&home_path)?;
        return Ok(home_path);
    }
    Ok(qitech_path)
}

fn determine_user_state_home() -> Result<String> {
    let state_home = env::var("XDG_STATE_HOME");
    let home = env::var("XDG_STATE_HOME");
    if state_home.is_ok() {
        return Ok(state_home.unwrap() + "/qitech");
    } else if home.is_ok() {
        return Ok(home.unwrap() + "/.local/state/qitech");
    };
    Err(Error::new(std::io::ErrorKind::NotFound, ERRMSG))
}
