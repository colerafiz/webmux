use anyhow::{Result, Context};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tokio::sync::RwLock;
use tracing::{error, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DotFile {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub modified: DateTime<Utc>,
    pub exists: bool,
    pub readable: bool,
    pub writable: bool,
    pub file_type: DotFileType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DotFileType {
    Shell,      // .bashrc, .zshrc, .profile
    Git,        // .gitconfig, .gitignore
    Vim,        // .vimrc, .vim/
    Tmux,       // .tmux.conf
    SSH,        // .ssh/config
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileVersion {
    pub timestamp: DateTime<Utc>,
    pub content: String,
    pub size: u64,
    pub hash: String,
}

pub struct DotFilesManager {
    // Store file history in memory for now (could be moved to DB later)
    history: RwLock<HashMap<String, Vec<FileVersion>>>,
    // Common dotfile paths relative to home directory
    common_dotfiles: Vec<(&'static str, DotFileType)>,
}

impl DotFilesManager {
    pub fn new() -> Self {
        Self {
            history: RwLock::new(HashMap::new()),
            common_dotfiles: vec![
                (".bashrc", DotFileType::Shell),
                (".zshrc", DotFileType::Shell),
                (".profile", DotFileType::Shell),
                (".bash_profile", DotFileType::Shell),
                (".bash_aliases", DotFileType::Shell),
                (".gitconfig", DotFileType::Git),
                (".gitignore_global", DotFileType::Git),
                (".vimrc", DotFileType::Vim),
                (".vim/vimrc", DotFileType::Vim),
                (".tmux.conf", DotFileType::Tmux),
                (".ssh/config", DotFileType::SSH),
                (".ssh/authorized_keys", DotFileType::SSH),
                (".ssh/known_hosts", DotFileType::SSH),
            ],
        }
    }

    /// List common dotfiles with their metadata
    pub async fn list_dotfiles(&self) -> Result<Vec<DotFile>> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
        
        info!("Listing dotfiles from home directory: {}", home_dir.display());
        
        let mut dotfiles = Vec::new();
        
        for (file_name, file_type) in &self.common_dotfiles {
            let file_path = home_dir.join(file_name);
            let path_str = file_path.to_string_lossy().to_string();
            
            let dotfile = if file_path.exists() {
                let metadata = fs::metadata(&file_path)?;
                let modified = metadata.modified()?
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_secs();
                
                DotFile {
                    name: file_name.to_string(),
                    path: path_str,
                    size: metadata.len(),
                    modified: DateTime::from_timestamp(modified as i64, 0)
                        .unwrap_or_else(|| Utc::now()),
                    exists: true,
                    readable: self.is_readable(&file_path),
                    writable: self.is_writable(&file_path),
                    file_type: file_type.clone(),
                }
            } else {
                DotFile {
                    name: file_name.to_string(),
                    path: path_str,
                    size: 0,
                    modified: Utc::now(),
                    exists: false,
                    readable: false,
                    writable: false,
                    file_type: file_type.clone(),
                }
            };
            
            dotfiles.push(dotfile);
        }
        
        info!("Listed {} dotfiles", dotfiles.len());
        Ok(dotfiles)
    }

    /// Read a dotfile's content with security validation
    pub async fn read_dotfile(&self, path: &str) -> Result<String> {
        let file_path = self.validate_and_resolve_path(path)?;
        
        // Debug logging
        info!("Reading dotfile - requested path: {}, resolved path: {}", path, file_path.display());
        
        // Check if file is readable
        if !self.is_readable(&file_path) {
            return Err(anyhow::anyhow!("File is not readable: {}", path));
        }
        
        // Read file content
        let content = fs::read_to_string(&file_path)
            .with_context(|| format!("Failed to read file: {}", path))?;
        
        info!("Read dotfile: {} ({} bytes) from {}", path, content.len(), file_path.display());
        
        // Check if this is .zshrc and log more details
        if path.contains("zshrc") {
            info!(".zshrc detected - checking content characteristics");
            let lines: Vec<&str> = content.lines().collect();
            info!("Total lines: {}", lines.len());
            info!("First non-comment line: {:?}", lines.iter().find(|l| !l.trim().starts_with('#') && !l.trim().is_empty()));
            info!("Contains 'Zim' setup: {}", content.contains("Zim"));
            info!("Contains 'autoload': {}", content.contains("autoload"));
        }
        
        // Log first 200 chars for debugging
        let preview = if content.len() > 200 { 
            format!("{}...", &content[..200])
        } else { 
            content.clone() 
        };
        info!("Content preview: {}", preview.replace('\n', "\\n"));
        
        Ok(content)
    }

    /// Write content to a dotfile with backup
    pub async fn write_dotfile(&self, path: &str, content: &str) -> Result<()> {
        let file_path = self.validate_and_resolve_path(path)?;
        
        // Check if file/directory is writable
        if file_path.exists() && !self.is_writable(&file_path) {
            return Err(anyhow::anyhow!("File is not writable: {}", path));
        }
        
        // Create backup if file exists
        if file_path.exists() {
            self.create_backup(&file_path).await?;
        }
        
        // Ensure parent directory exists
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Write new content
        fs::write(&file_path, content)
            .with_context(|| format!("Failed to write file: {}", path))?;
        
        info!("Wrote dotfile: {} ({} bytes)", path, content.len());
        Ok(())
    }

    /// Create a backup of a file
    async fn create_backup(&self, file_path: &Path) -> Result<()> {
        let content = fs::read_to_string(file_path)?;
        let metadata = fs::metadata(file_path)?;
        
        let version = FileVersion {
            timestamp: Utc::now(),
            content: content.clone(),
            size: metadata.len(),
            hash: self.calculate_hash(&content),
        };
        
        let path_str = file_path.to_string_lossy().to_string();
        let mut history = self.history.write().await;
        let versions = history.entry(path_str.clone()).or_insert_with(Vec::new);
        
        // Keep only last 10 versions
        if versions.len() >= 10 {
            versions.remove(0);
        }
        
        versions.push(version);
        info!("Created backup for: {}", path_str);
        Ok(())
    }

    /// Get version history for a file
    pub async fn get_file_history(&self, path: &str) -> Result<Vec<FileVersion>> {
        let file_path = self.validate_and_resolve_path(path)?;
        let path_str = file_path.to_string_lossy().to_string();
        
        let history = self.history.read().await;
        Ok(history.get(&path_str).cloned().unwrap_or_default())
    }

    /// Restore a file from a specific version
    pub async fn restore_version(&self, path: &str, timestamp: DateTime<Utc>) -> Result<()> {
        let file_path = self.validate_and_resolve_path(path)?;
        let path_str = file_path.to_string_lossy().to_string();
        
        // Get the content to restore
        let content_to_restore = {
            let history = self.history.read().await;
            let versions = history.get(&path_str)
                .ok_or_else(|| anyhow::anyhow!("No history found for file"))?;
            
            let version = versions.iter()
                .find(|v| v.timestamp == timestamp)
                .ok_or_else(|| anyhow::anyhow!("Version not found"))?;
            
            version.content.clone()
        }; // Read lock is released here
        
        // Write the old version content
        self.write_dotfile(path, &content_to_restore).await?;
        
        Ok(())
    }

    /// Validate and resolve file path - simplified for personal use
    fn validate_and_resolve_path(&self, path: &str) -> Result<PathBuf> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
        
        // Resolve the path
        let file_path = if path.starts_with("~/") {
            home_dir.join(&path[2..])
        } else if path.starts_with('/') {
            PathBuf::from(path)
        } else {
            home_dir.join(path)
        };
        
        // Canonicalize to resolve symlinks and relative paths
        let canonical_path = if file_path.exists() {
            file_path.canonicalize()?
        } else {
            // For non-existent files, canonicalize the parent and append filename
            if let Some(parent) = file_path.parent() {
                if parent.exists() {
                    let canonical_parent = parent.canonicalize()?;
                    canonical_parent.join(file_path.file_name().unwrap_or_default())
                } else {
                    file_path
                }
            } else {
                file_path
            }
        };
        
        Ok(canonical_path)
    }

    /// Check if a file is readable
    fn is_readable(&self, path: &Path) -> bool {
        fs::File::open(path).is_ok()
    }

    /// Check if a file is writable
    fn is_writable(&self, path: &Path) -> bool {
        use std::fs::OpenOptions;
        
        if path.exists() {
            OpenOptions::new()
                .write(true)
                .open(path)
                .is_ok()
        } else if let Some(parent) = path.parent() {
            // Check if parent directory is writable
            parent.exists() && {
                let test_file = parent.join(".webmux_write_test");
                let result = fs::write(&test_file, "test").is_ok();
                let _ = fs::remove_file(&test_file);
                result
            }
        } else {
            false
        }
    }

    /// Calculate a simple hash for version tracking
    fn calculate_hash(&self, content: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Get default templates for common config files
    pub fn get_templates(&self) -> Vec<DotFileTemplate> {
        vec![
            DotFileTemplate {
                name: "Basic .bashrc".to_string(),
                file_type: DotFileType::Shell,
                description: "Basic bash configuration with common aliases".to_string(),
                content: BASHRC_TEMPLATE.to_string(),
            },
            DotFileTemplate {
                name: "Basic .vimrc".to_string(),
                file_type: DotFileType::Vim,
                description: "Basic vim configuration with sensible defaults".to_string(),
                content: VIMRC_TEMPLATE.to_string(),
            },
            DotFileTemplate {
                name: "Basic .gitconfig".to_string(),
                file_type: DotFileType::Git,
                description: "Git configuration with useful aliases".to_string(),
                content: GITCONFIG_TEMPLATE.to_string(),
            },
            DotFileTemplate {
                name: "Basic .tmux.conf".to_string(),
                file_type: DotFileType::Tmux,
                description: "Tmux configuration with improved defaults".to_string(),
                content: TMUX_CONF_TEMPLATE.to_string(),
            },
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DotFileTemplate {
    pub name: String,
    pub file_type: DotFileType,
    pub description: String,
    pub content: String,
}

lazy_static::lazy_static! {
    pub static ref DOTFILES_MANAGER: DotFilesManager = DotFilesManager::new();
}

// Template constants
const BASHRC_TEMPLATE: &str = include_str!("templates/bashrc.template");
const VIMRC_TEMPLATE: &str = include_str!("templates/vimrc.template");
const GITCONFIG_TEMPLATE: &str = include_str!("templates/gitconfig.template");
const TMUX_CONF_TEMPLATE: &str = include_str!("templates/tmux.conf.template");