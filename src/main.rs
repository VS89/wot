use wot::zip_directory;
// https://github.com/clap-rs/clap?tab=readme-ov-file
// https://docs.rs/clap/latest/clap/

fn main() {
    let result = zip_directory("/Users/valentins/Desktop/test_allure_report");
    println!("{:?}", result);
}
