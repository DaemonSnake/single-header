fn main() {
    if let Err(_) = which::which("cpp") {
        println!("cargo:warning=`cpp` not found on system, please install it");
        std::process::exit(1);
    }
}
