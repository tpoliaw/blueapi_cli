use crate::cli::AuthCommand;
use crate::entities::AuthConfig;

pub fn login(cmd: AuthCommand, conf: Option<AuthConfig>) {
    println!("{cmd:?}");
    println!("{conf:?}");
    todo!()
}
