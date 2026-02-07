use std::fs;
use std::path::Path;
use crate::traits::{Init, InitError, Logger};

pub fn sanitize_resource_name(raw: &str) -> String {
    let s: String = raw
        .to_lowercase()
        .chars()
        .map(|c| if c == ' ' || c == '_' { '-' } else { c })
        .filter(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '-')
        .collect();

    let s = s.trim_matches('-').to_string();

    // collapse consecutive hyphens
    let mut result = String::with_capacity(s.len());
    let mut prev_hyphen = false;
    for c in s.chars() {
        if c == '-' {
            if !prev_hyphen {
                result.push(c);
            }
            prev_hyphen = true;
        } else {
            result.push(c);
            prev_hyphen = false;
        }
    }

    // truncate to 63 chars, trim trailing hyphens
    if result.len() > 63 {
        result.truncate(63);
    }
    result.trim_end_matches('-').to_string()
}

pub fn validate_resource_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("name must not be empty".to_string());
    }
    if name.len() > 63 {
        return Err(format!(
            "name must be no more than 63 characters, got {}",
            name.len()
        ));
    }
    if !name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        return Err(
            "name must contain only lowercase alphanumeric characters or '-'".to_string(),
        );
    }
    let first = name.chars().next().unwrap();
    if !first.is_ascii_alphanumeric() {
        return Err("name must start with an alphanumeric character".to_string());
    }
    let last = name.chars().last().unwrap();
    if !last.is_ascii_alphanumeric() {
        return Err("name must end with an alphanumeric character".to_string());
    }
    Ok(())
}

pub struct InitImpl {
    logger: Box<dyn Logger>,
}

impl InitImpl {
    pub fn new(logger: Box<dyn Logger>) -> Self {
        InitImpl { logger }
    }

    fn resolve_name(path: &Path, name: Option<&str>) -> Result<String, InitError> {
        match name {
            Some(n) => {
                validate_resource_name(n).map_err(|reason| InitError::InvalidResourceName {
                    name: n.to_string(),
                    reason,
                })?;
                Ok(n.to_string())
            }
            None => {
                let absolute = path.canonicalize().map_err(|_| {
                    InitError::InvalidDirectoryName(path.display().to_string())
                })?;
                let dir_name = absolute
                    .file_name()
                    .and_then(|n| n.to_str())
                    .ok_or_else(|| InitError::InvalidDirectoryName(path.display().to_string()))?;

                let sanitized = sanitize_resource_name(dir_name);
                validate_resource_name(&sanitized).map_err(|reason| {
                    InitError::InvalidResourceName {
                        name: sanitized.clone(),
                        reason,
                    }
                })?;
                Ok(sanitized)
            }
        }
    }
}

pub fn build_project_yaml(name: &str) -> Result<String, String> {
    validate_resource_name(name)?;
    Ok(format!("\
apiVersion: project.dbloada.io/v1
kind: DBLoadaProject
metadata:
   name: {name}
spec:
   # spec goes here empty for now
"))
}

impl Init for InitImpl {
    fn init(&self, path: &Path, name: Option<&str>) -> Result<(), InitError> {
        if !path.is_dir() {
            return Err(InitError::DirectoryNotFound(path.display().to_string()));
        }

        let project_name = Self::resolve_name(path, name)?;

        let yaml = build_project_yaml(&project_name).map_err(|reason| {
            InitError::InvalidResourceName {
                name: project_name.clone(),
                reason,
            }
        })?;

        let file_path = path.join("dbloada.yaml");
        fs::write(&file_path, &yaml)?;

        self.logger.info(&format!("created {}", file_path.display()));
        Ok(())
    }
}
