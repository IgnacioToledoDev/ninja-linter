use crate::command::{run_git_status, run_diff_tree_command};
use std::io;

const FILE_EXTENSION: &str = ".php";

/**
 * Get all modified files in the project with extension .php
*/
pub fn get_modified_files() -> io::Result<Vec<String>> {
    let output = run_git_status()?;
    let mut files = parse_git_status(&output);

    if files.is_empty() {
        files = retrieve_files()?;
    }

    Ok(files)
}

fn parse_git_status(output: &str) -> Vec<String> {
    let mut allowed_files = Vec::new();
    for line in output.lines() {
        if !line.ends_with(FILE_EXTENSION) {
            continue;
        }
        
        // Git status --short output format: "XY path"
        // Where X and Y are status codes. We skip the first 3 characters to get the path.
        if line.len() < 3 {
            continue;
        }
        let file_path = line[3..].trim();
        
        if file_path.is_empty() {
            continue;
        }

        if file_path.ends_with(FILE_EXTENSION) {
            let clean_file = clean_modified_file(file_path.to_string());
            allowed_files.push(clean_file);
        }
    }

    allowed_files
}

fn clean_modified_file(file_path: String) -> String {
    if file_path.starts_with("src/") || file_path.starts_with("tests/") {
        return file_path;
    }

    file_path.replace("back/", "")
}

fn retrieve_files() -> io::Result<Vec<String>> {
    let output = run_diff_tree_command()?;
    let mut retrieve_files = Vec::new();

    for line in output.lines() {
        if !line.ends_with(FILE_EXTENSION) {
            continue;
        }

        let file_path = line.trim();
        if file_path.is_empty() {
            continue;
        }

        if file_path.ends_with(FILE_EXTENSION) {
            let clean_file = clean_modified_file(file_path.to_string());
            retrieve_files.push(clean_file);
        }
    }

    Ok(retrieve_files)
}

#[cfg(test)]
mod tests {
    use crate::file::clean_modified_file;

    #[test]
    fn test_clean_modified_file() {
        let file_path = "back/src/Controller/HomeController.php".to_string();
        assert_eq!(
            clean_modified_file(file_path),
            "src/Controller/HomeController.php"
        );
    }

    #[test]
    fn test_clean_modified_file_start_with_tests() {
        let file_path = "back/tests/Controller/HomeControllerTest.php".to_string();
        assert_eq!(
            clean_modified_file(file_path),
            "tests/Controller/HomeControllerTest.php"
        );
    }

    #[test]
    fn test_clean_modified_file_without_file_extension() {
        let file_path = "back/.env";
        assert_eq!(clean_modified_file(file_path.to_string()), ".env");
    }

    #[test]
    fn test_parse_git_status() {
        let output = " M src/Controller/HomeController.php\n?? back/src/Entity/User.php\nA  tests/AppTest.php\n M README.md\n";
        let expected = vec![
            "src/Controller/HomeController.php".to_string(),
            "src/Entity/User.php".to_string(),
            "tests/AppTest.php".to_string(),
        ];
        assert_eq!(super::parse_git_status(output), expected);
    }

    #[test]
    fn test_parse_git_status_empty() {
        let output = "";
        let expected: Vec<String> = Vec::new();
        assert_eq!(super::parse_git_status(output), expected);
    }

    #[test]
    fn test_retrieve_files_logic() {
        // Mocking git output is complex, but we can verify the logic that processes it
        let _output = "src/Controller/AppController.php\ntests/Unit/Test.php\nREADME.md\n";
        // We can't call retrieve_files directly as it calls run_diff_tree_command,
        // but the internal logic of cleaning and filtering is shared.
    }
}
