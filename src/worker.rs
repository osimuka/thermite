use std::process::Command;
use crate::task::Task;

pub fn execute_task(task: &Task) -> Result<String, String> {
    match task.language.as_str() {
        "python" => {
            let output = Command::new("python")
                .arg("-c")
                .arg(&task.script)
                .output()
                .expect("Failed to execute script");

            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            } else {
                Err(String::from_utf8_lossy(&output.stderr).to_string())
            }
        },
        // Add more languages here
        _ => Err(format!("Unsupported language: {}", task.language)),
    }
}
