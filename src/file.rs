use crate::command::run_git_status;

const FILE_EXTENSION: &str = ".php";

/**
 * Get all modified files in the project with extension .php
*/
pub fn get_modified_files() -> Vec<String> {
    let output = run_git_status();

    // TODO: This can be refactoring using stdout
    // TODO: Review this later to validate that it belongs to the Ninja Excels project
    let mut allowed_files = Vec::new();
    for file_path in output.split('\n') {
        if file_path.is_empty() {
            continue;
        }

        if file_path.ends_with(&FILE_EXTENSION) {
            let clean_file = clean_modified_file(file_path.to_string());
            allowed_files.push(clean_file);
        }
    }

    allowed_files
}

fn clean_modified_file(file_path: String) -> String {
    if file_path.starts_with("src/") {
        return file_path;
    }

    file_path.replace("back/", "")
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
    fn test_clean_modified_file_without_file_extension() {
        let file_path = "back/.env";
        assert_eq!(clean_modified_file(file_path.to_string()), ".env");
    }
}
