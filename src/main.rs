mod runtime;

use runtime::get_runtimes;

fn main() {
    let runtimes = get_runtimes();

    for runtime in &runtimes {
        println!("{:?}", runtime);
    }
}
