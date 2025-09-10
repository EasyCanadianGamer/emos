// File System Service for Microkernel
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};
use lazy_static::lazy_static;
use spin::Mutex;

/// File System Service - Handles file operations
pub struct FileSystemService {
    next_inode: AtomicU64,
    files: BTreeMap<u64, FileEntry>,
    directories: BTreeMap<u64, DirectoryEntry>,
    current_directory: u64,
}

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub inode: u64,
    pub name: String,
    pub size: usize,
    pub data: Vec<u8>,
    pub permissions: FilePermissions,
    pub created_at: u64,
    pub modified_at: u64,
}

#[derive(Debug, Clone)]
pub struct DirectoryEntry {
    pub inode: u64,
    pub name: String,
    pub parent: Option<u64>,
    pub children: Vec<u64>,
    pub created_at: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilePermissions {
    ReadOnly,
    WriteOnly,
    ReadWrite,
    Execute,
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
}

impl FileSystemService {
    pub fn new() -> Self {
        let mut service = Self {
            next_inode: AtomicU64::new(1),
            files: BTreeMap::new(),
            directories: BTreeMap::new(),
            current_directory: 0,
        };
        
        // Create root directory
        service.create_root_directory();
        service
    }

    fn create_root_directory(&mut self) {
        let root_inode = self.next_inode.fetch_add(1, Ordering::Relaxed);
        let root_dir = DirectoryEntry {
            inode: root_inode,
            name: String::from("/"),
            parent: None,
            children: Vec::new(),
            created_at: 0, // System boot time
        };
        self.directories.insert(root_inode, root_dir);
        self.current_directory = root_inode;
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
            for &child_inode in &current_dir.children {
                if let Some(file) = self.files.get(&child_inode) {
                    if file.name == name {
                        return Err(FileSystemError::FileExists);
                    }
                }
            }
        }

        let inode = self.next_inode.fetch_add(1, Ordering::Relaxed);
        let file = FileEntry {
            inode,
            name: String::from(name),
            size: 0,
            data: Vec::new(),
            permissions,
            created_at: 0, // System time
            modified_at: 0,
        };

        self.files.insert(inode, file);
        
        // Add to current directory
        if let Some(current_dir) = self.directories.get_mut(&self.current_directory) {
            current_dir.children.push(inode);
        }

        Ok(inode)
    }

    /// Create a new directory
    pub fn create_directory(&mut self, name: &str) -> Result<u64, FileSystemError> {
        if name.is_empty() || name.contains('/') {
            return Err(FileSystemError::InvalidPath);
        }

        // Check if directory already exists
        if let Some(current_dir) = self.directories.get(&self.current_directory) {
            for &child_inode in &current_dir.children {
                if let Some(dir) = self.directories.get(&child_inode) {
                    if dir.name == name {
                        return Err(FileSystemError::FileExists);
                    }
                }
            }
        }

        let inode = self.next_inode.fetch_add(1, Ordering::Relaxed);
        let directory = DirectoryEntry {
            inode,
            name: String::from(name),
            parent: Some(self.current_directory),
            children: Vec::new(),
            created_at: 0, // System time
        };

        self.directories.insert(inode, directory);
        
        // Add to current directory
        if let Some(current_dir) = self.directories.get_mut(&self.current_directory) {
            current_dir.children.push(inode);
        }

        Ok(inode)
    }

    /// Write data to a file
    pub fn write_file(
        &mut self,
        inode: u64,
        data: &[u8],
    ) -> Result<usize, FileSystemError> {
        if let Some(file) = self.files.get_mut(&inode) {
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
    pub fn read_file(&self, inode: u64) -> Result<Vec<u8>, FileSystemError> {
        if let Some(file) = self.files.get(&inode) {
            if file.permissions == FilePermissions::WriteOnly {
                return Err(FileSystemError::PermissionDenied);
            }
            Ok(file.data.clone())
        } else {
            Err(FileSystemError::FileNotFound)
        }
    }

    /// Delete a file
    pub fn delete_file(&mut self, inode: u64) -> Result<(), FileSystemError> {
        if let Some(_file) = self.files.remove(&inode) {
            // Remove from parent directory
            if let Some(current_dir) = self.directories.get_mut(&self.current_directory) {
                current_dir.children.retain(|&child| child != inode);
            }
            Ok(())
        } else {
            Err(FileSystemError::FileNotFound)
        }
    }

    /// List files in current directory
    pub fn list_files(&self) -> Vec<(String, bool)> {
        let mut result = Vec::new();
        
        if let Some(current_dir) = self.directories.get(&self.current_directory) {
            for &child_inode in &current_dir.children {
                if let Some(file) = self.files.get(&child_inode) {
                    result.push((file.name.clone(), false)); // false = file
                } else if let Some(dir) = self.directories.get(&child_inode) {
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
            for &child_inode in &current_dir.children {
                if let Some(dir) = self.directories.get(&child_inode) {
                    if dir.name == name {
                        self.current_directory = child_inode;
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
}

lazy_static! {
    pub static ref FILESYSTEM_SERVICE: Mutex<FileSystemService> = Mutex::new(FileSystemService::new());
}

/// File system service API functions
pub fn create_file(name: &str, permissions: FilePermissions) -> Result<u64, FileSystemError> {
    FILESYSTEM_SERVICE.lock().create_file(name, permissions)
}

pub fn write_file(inode: u64, data: &[u8]) -> Result<usize, FileSystemError> {
    FILESYSTEM_SERVICE.lock().write_file(inode, data)
}

pub fn read_file(inode: u64) -> Result<Vec<u8>, FileSystemError> {
    FILESYSTEM_SERVICE.lock().read_file(inode)
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