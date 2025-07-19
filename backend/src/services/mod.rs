
use pam::Authenticator;
use std::process::Command;
use users;
pub mod jwt;


pub fn pam_auth(username: &str, password: &str) -> Result<bool, String> {
    let mut auth = Authenticator::with_password("login").map_err(|e| e.to_string())?;

    let handler = auth.get_handler();
    handler.set_credentials(username, password);

    auth.authenticate().map_err(|e| e.to_string())?;

    if let Some(user) = users::get_user_by_name(username) {
        let uid = user.uid();
        if uid == 0 || is_user_in_sudo_group(username) {
            return Ok(true);
        }
    }

    Ok(false)
}

fn is_user_in_sudo_group(username: &str) -> bool {
    if let Ok(output) = Command::new("groups").arg(username).output() {
        let group_list = String::from_utf8_lossy(&output.stdout);
        return group_list.split_whitespace().any(|g| g == "sudo");
    }
    false
}
