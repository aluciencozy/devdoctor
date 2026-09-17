use std::env;

fn main() {
    const PATH_ENV_KEY: &str = "PATH";
    match env::var_os(PATH_ENV_KEY) {
        Some(paths) => {
            for path in env::split_paths(&paths) {
                println!("{}", path.display());
            }
        }
        None => println!("{PATH_ENV_KEY} is not defined in the environment"),
    }
}
