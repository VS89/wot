use wot::zip_directory;

fn main() {
    let result = zip_directory("/Users/valentins/Desktop/test_allure_report");
    println!("{:?}", result);
}
