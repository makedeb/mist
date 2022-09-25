use std::os::unix::fs::PermissionsExt;

fn main() {
    let file = std::fs::File::open("target/debug/mist").unwrap();
    println!("{:o}", file.metadata().unwrap().permissions().mode());
}
