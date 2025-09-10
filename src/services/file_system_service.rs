// FAT-inspired File System Service for Microkernel (no_std compatible)
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};
use lazy_static::lazy_static;
use spin::Mutex;

/// FAT-inspired File System Service - Handles file operations
/// This is a simplified implementation inspired by FAT filesystem structure
pub struct FileSystemService {
    next_cluster: AtomicU64,
    files: BTreeMap<u64, FileEntry>,
    directories: BTreeMap<u64, DirectoryEntry>,
    current_directory: u64,
    fat_table: BTreeMap<u64, u64>, // Cluster chain mapping
}

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub cluster: u64,        // First cluster (like FAT)
    pub name: String,
    pub size: usize,
    pub data: Vec<u8>,
    pub permissions: FilePermissions,
    pub created_at: u64,
    pub modified_at: u64,
    pub attributes: FileAttributes,
}

#[derive(Debug, Clone)]
pub struct DirectoryEntry {
    pub cluster: u64,
    pub name: String,
    pub parent: Option<u64>,
    pub children: Vec<u64>,
    pub created_at: u64,
    pub attributes: FileAttributes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilePermissions {
    ReadOnly,
    WriteOnly,
    ReadWrite,
    Execute,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileAttributes {
    Archive = 0x20,
    Directory = 0x10,
    VolumeLabel = 0x08,
    System = 0x04,
    Hidden = 0x02,
    ReadOnly = 0x01,
}

#[derive(Debug)]
pub enum FileSystemError {
    FileNotFound,
    DirectoryNotFound,
    PermissionDenied,
    FileExists,
    DirectoryNotEmpty,
    InvalidPath,
    OutOfSpace,
    InvalidCluster,
    ClusterChainError,
}

impl FileSystemService {
    pub fn new() -> Self {
        let mut service = Self {
            next_cluster: AtomicU64::new(2), // Start from cluster 2 (like FAT)
            files: BTreeMap::new(),
            directories: BTreeMap::new(),
            current_directory: 0,
            fat_table: BTreeMap::new(),
        };
        
        // Create root directory (cluster 0)
        service.create_root_directory();
        service
    }

    fn create_root_directory(&mut self) {
        let root_cluster = 0;
        let root_dir = DirectoryEntry {
            cluster: root_cluster,
            name: String::from("/"),
            parent: None,
            children: Vec::new(),
            created_at: 0, // System boot time
            attributes: FileAttributes::Directory,
        };
        self.directories.insert(root_cluster, root_dir);
        self.current_directory = root_cluster;
    }

    /// Allocate a new cluster (FAT-style)
    fn allocate_cluster(&mut self) -> u64 {
        let cluster = self.next_cluster.fetch_add(1, Ordering::Relaxed);
        self.fat_table.insert(cluster, 0xFFFFFFFF); // End of chain marker
        cluster
    }

    /// Create a new file
    pub fn create_file(
        &mut self,
        name: &str,
        permissions: FilePermissions,
    ) -> Result<u64, FileSystemError> {
        if name.is_empty() || name.contains('/') {
            return Err(FileSystemError::InvalidPath);
        }

        // Check if file already exists in current directory
        if let Some(current_dir) = self.directories.get(&self.current_directory) {
            for &child_cluster in &current_dir.children {
                if let Some(file) = self.files.get(&child_cluster) {
                    if file.name == name {
                        return Err(FileSystemError::FileExists);
                    }
                }
            }
        }

        let cluster = self.allocate_cluster();
        let file = FileEntry {
            cluster,
            name: String::from(name),
            size: 0,
            data: Vec::new(),
            permissions,
            created_at: 0, // System time
            modified_at: 0,
            attributes: FileAttributes::Archive,
        };

        self.files.insert(cluster, file);
        
        // Add to current directory
        if let Some(current_dir) = self.directories.get_mut(&self.current_directory) {
            current_dir.children.push(cluster);
        }

        Ok(cluster)
    }

    /// Create a new directory
    pub fn create_directory(&mut self, name: &str) -> Result<u64, FileSystemError> {
        if name.is_empty() || name.contains('/') {
            return Err(FileSystemError::InvalidPath);
        }

        // Check if directory already exists
        if let Some(current_dir) = self.directories.get(&self.current_directory) {
            for &child_cluster in &current_dir.children {
                if let Some(dir) = self.directories.get(&child_cluster) {
                    if dir.name == name {
                        return Err(FileSystemError::FileExists);
                    }
                }
            }
        }

        let cluster = self.allocate_cluster();
        let directory = DirectoryEntry {
            cluster,
            name: String::from(name),
            parent: Some(self.current_directory),
            children: Vec::new(),
            created_at: 0, // System time
            attributes: FileAttributes::Directory,
        };

        self.directories.insert(cluster, directory);
        
        // Add to current directory
        if let Some(current_dir) = self.directories.get_mut(&self.current_directory) {
            current_dir.children.push(cluster);
        }

        Ok(cluster)
    }

    /// Write data to a file
    pub fn write_file(
        &mut self,
        cluster: u64,
        data: &[u8],
    ) -> Result<usize, FileSystemError> {
        if let Some(file) = self.files.get_mut(&cluster) {
            if file.permissions == FilePermissions::ReadOnly {
                return Err(FileSystemError::PermissionDenied);
            }

            file.data.clear();
            file.data.extend_from_slice(data);
            file.size = data.len();
            file.modified_at = 0; // System time
            Ok(data.len())
        } else {
            Err(FileSystemError::FileNotFound)
        }
    }

    /// Read data from a file
    pub fn read_file(&self, cluster: u64) -> Result<Vec<u8>, FileSystemError> {
        if let Some(file) = self.files.get(&cluster) {
            if file.permissions == FilePermissions::WriteOnly {
                return Err(FileSystemError::PermissionDenied);
            }
            Ok(file.data.clone())
        } else {
            Err(FileSystemError::FileNotFound)
        }
    }

    /// Delete a file
    pub fn delete_file(&mut self, cluster: u64) -> Result<(), FileSystemError> {
        if let Some(_file) = self.files.remove(&cluster) {
            // Remove from parent directory
            if let Some(current_dir) = self.directories.get_mut(&self.current_directory) {
                current_dir.children.retain(|&child| child != cluster);
            }
            // Free the cluster (FAT-style)
            self.fat_table.remove(&cluster);
            Ok(())
        } else {
            Err(FileSystemError::FileNotFound)
        }
    }

    /// List files in current directory
    pub fn list_files(&self) -> Vec<(String, bool)> {
        let mut result = Vec::new();
        
        if let Some(current_dir) = self.directories.get(&self.current_directory) {
            for &child_cluster in &current_dir.children {
                if let Some(file) = self.files.get(&child_cluster) {
                    result.push((file.name.clone(), false)); // false = file
                } else if let Some(dir) = self.directories.get(&child_cluster) {
                    result.push((dir.name.clone(), true)); // true = directory
                }
            }
        }
        
        result
    }

    /// Change current directory
    pub fn change_directory(&mut self, name: &str) -> Result<(), FileSystemError> {
        if name == ".." {
            if let Some(current_dir) = self.directories.get(&self.current_directory) {
                if let Some(parent) = current_dir.parent {
                    self.current_directory = parent;
                    return Ok(());
                }
            }
            return Err(FileSystemError::DirectoryNotFound);
        }

        if let Some(current_dir) = self.directories.get(&self.current_directory) {
            for &child_cluster in &current_dir.children {
                if let Some(dir) = self.directories.get(&child_cluster) {
                    if dir.name == name {
                        self.current_directory = child_cluster;
                        return Ok(());
                    }
                }
            }
        }
        
        Err(FileSystemError::DirectoryNotFound)
    }

    /// Get current working directory path
    pub fn get_current_path(&self) -> String {
        let mut path = String::new();
        let mut current = self.current_directory;
        
        while let Some(dir) = self.directories.get(&current) {
            if dir.name == "/" {
                path.insert_str(0, "/");
                break;
            } else {
                path.insert_str(0, &format!("{}/", dir.name));
                current = dir.parent.unwrap_or(0);
            }
        }
        
        path
    }

    /// Get FAT table information (for debugging)
    pub fn get_fat_info(&self) -> (usize, usize) {
        (self.fat_table.len(), self.files.len() + self.directories.len())
    }

    /// Check if a cluster is allocated
    pub fn is_cluster_allocated(&self, cluster: u64) -> bool {
        self.fat_table.contains_key(&cluster) || cluster == 0
    }
}

lazy_static! {
    pub static ref FILESYSTEM_SERVICE: Mutex<FileSystemService> = Mutex::new(FileSystemService::new());
}

/// File system service API functions
pub fn create_file(name: &str, permissions: FilePermissions) -> Result<u64, FileSystemError> {
    FILESYSTEM_SERVICE.lock().create_file(name, permissions)
}

pub fn write_file(cluster: u64, data: &[u8]) -> Result<usize, FileSystemError> {
    FILESYSTEM_SERVICE.lock().write_file(cluster, data)
}

pub fn read_file(cluster: u64) -> Result<Vec<u8>, FileSystemError> {
    FILESYSTEM_SERVICE.lock().read_file(cluster)
}

pub fn list_files() -> Vec<(String, bool)> {
    FILESYSTEM_SERVICE.lock().list_files()
}

pub fn change_directory(name: &str) -> Result<(), FileSystemError> {
    FILESYSTEM_SERVICE.lock().change_directory(name)
}

pub fn get_current_path() -> String {
    FILESYSTEM_SERVICE.lock().get_current_path()
}

/// Initialize the FAT-inspired filesystem
pub fn init_fat_filesystem() -> Result<(), FileSystemError> {
    // Filesystem is already initialized in the lazy_static
    Ok(())
}