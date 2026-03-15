use std::env;
use std::fs;
use std::io::Cursor;
use std::path::Path;
use reqwest::blocking::Client;
use scraper::{Html, Selector};

fn main() {
    // Define the input/output directory
    dotenvy::dotenv().ok();

    let test_dir = "tests/inout";
    
    // Fetch from env (either from .env file or system env)
    let problem_id = env::var("CSES_PROBLEM_ID").expect("CSES_PROBLEM_ID not set");
    let username = env::var("CSES_USER").expect("CSES_USER not set");
    let password = env::var("CSES_PASS").expect("CSES_PASS not set");

    if !Path::new(test_dir).exists() || fs::read_dir(test_dir).map(|d| d.count()).unwrap_or(0) == 0 {
        fs::create_dir_all(test_dir).unwrap();
        download_tests(&problem_id, &username, &password, test_dir);
    }

    let out_dir = env::var("OUT_DIR").unwrap();
    let generated_test_file = Path::new(&out_dir).join("generated_tests.rs");

    // Collect all .in and .out file pairs
    let entries = fs::read_dir(test_dir).expect("Failed to read test directory");
    let mut test_cases = vec![];

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("in") {
            let output_file = path.with_extension("out");
            if output_file.exists() {
                test_cases.push((path, output_file));
            } else {
                panic!("No matching .out file for {}", path.display());
            }
        }
    }

    // Load the test function template from the file
    let test_template = fs::read_to_string("test_template.txt")
        .expect("Failed to read test template file");

    // Generate Rust test code
    let mut test_code = String::new();
    test_code.push_str("use super::*;\n\n");

    for (input_file, output_file) in test_cases {
        let test_name = input_file
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .replace("-", "_");

        let input_content = fs::read_to_string(&input_file).expect("Failed to read input file");
        let expected_output = fs::read_to_string(&output_file).expect("Failed to read output file");

        // Format the test template with actual values
        test_code.push_str(&test_template
            .replace("{test_name}", &test_name)
            .replace("{input_content}", &escape_string(&input_content))
            .replace("{expected_output}", &escape_string(&expected_output))
            .replace("{input_file}", &input_file.display().to_string()));
    }

    // Write the generated test code to a file
    fs::write(&generated_test_file, test_code).expect("Failed to write generated test file");
    println!("cargo:rerun-if-changed={}", test_dir);
}

/// Helper function to escape strings for use in Rust raw string literals
fn escape_string(s: &str) -> String {
    s.replace("\"", "\\\"")
}

fn download_tests(id: &str, user: &str, pass: &str, target: &str) {
    let client = Client::builder()
        .cookie_store(true)
        .user_agent("Mozilla/5.0 (Rust Build Script)")
        .build()
        .unwrap();

    // LOGIN
    let login_page = client.get("https://cses.fi/login").send().unwrap().text().unwrap();
    let token = extract_token(&login_page);

    let mut login_form = std::collections::HashMap::new();
    login_form.insert("csrf_token", token);
    login_form.insert("nick", user.to_string());
    login_form.insert("pass", pass.to_string());
    client.post("https://cses.fi/login").form(&login_form).send().unwrap();

    // DOWNLOAD
    let task_url = format!("https://cses.fi/problemset/task/{}", id);
    let task_page = client.get(&task_url).send().unwrap().text().unwrap();
    let dl_token = extract_token(&task_page);

    let mut dl_form = std::collections::HashMap::new();
    dl_form.insert("csrf_token", dl_token);
    dl_form.insert("download", "true".to_string());

    let mut response = client.post(&task_url).form(&dl_form).send().expect("Download failed");

    // UNZIP
    let mut content = Vec::new();
    std::io::copy(&mut response, &mut content).unwrap();
    let mut archive = zip::ZipArchive::new(Cursor::new(content)).unwrap();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let outpath = Path::new(target).join(file.name());
        let mut outfile = fs::File::create(&outpath).unwrap();
        std::io::copy(&mut file, &mut outfile).unwrap();
    }
}

fn extract_token(html: &str) -> String {
    let doc = Html::parse_document(html);
    let sel = Selector::parse("input[name='csrf_token']").unwrap();
    doc.select(&sel).next().unwrap().value().attr("value").unwrap().to_string()
}
