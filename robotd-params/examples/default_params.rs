fn main() {
    println!("# Local schema example, not a hardware deployment profile.");
    println!(
        "{}",
        toml::to_string_pretty(&robotd_params::Params::default()).unwrap()
    );
}
