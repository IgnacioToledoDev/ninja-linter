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
    for file in output.split('\n') {
        if file.is_empty() {
            continue;
        }

        if file.ends_with(&FILE_EXTENSION) {
            allowed_files.push(file.to_string());
        }
    }

    allowed_files
}
