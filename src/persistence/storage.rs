use super::{header::FileHeader, PersistenceError};
use crc32fast::Hasher;
use std::sync::{Arc, RwLock};

/// Abstraction for persistent storage backend.
///
/// Enables swappable backends for File (native) and IndexedDB (WASM).
#[cfg(not(target_arch = "wasm32"))]
pub trait StorageBackend: Send + Sync {
    /// Appends data to the end of the storage.
    ///
    /// Used for Write-Ahead Log (WAL) operations.
    ///
    /// # Errors
    /// Returns `PersistenceError::Io` if writing fails.
    fn append(&mut self, data: &[u8]) -> Result<(), PersistenceError>;

    /// Reads the entire storage content.
    ///
    /// Used for recovery/replay.
    ///
    /// # Errors
    /// Returns `PersistenceError::Io` if reading fails.
    fn read(&self) -> Result<Vec<u8>, PersistenceError>;

    /// Atomically writes the provided bytes identified by `key`.
    ///
    /// Implementations should ensure that either the previous value remains
    /// intact or the new value is fully committed. Backends that represent a
    /// single file may choose to ignore `key` and always operate on their
    /// configured path.
    ///
    /// # Errors
    /// Returns `PersistenceError::Io` if writing or renaming fails.
    fn atomic_write(&self, key: &str, data: &[u8]) -> Result<(), PersistenceError>;
}

/// Abstraction for persistent storage backend.
///
/// Enables swappable backends for File (native) and IndexedDB (WASM).
#[cfg(target_arch = "wasm32")]
pub trait StorageBackend {
    /// Appends data to the end of the storage.
    ///
    /// Used for Write-Ahead Log (WAL) operations.
    ///
    /// # Errors
    /// Returns `PersistenceError::Io` if writing fails.
    fn append(&mut self, data: &[u8]) -> Result<(), PersistenceError>;

    /// Reads the entire storage content.
    ///
    /// Used for recovery/replay.
    ///
    /// # Errors
    /// Returns `PersistenceError::Io` if reading fails.
    fn read(&self) -> Result<Vec<u8>, PersistenceError>;

    /// Atomically writes the provided bytes identified by `key`.
    ///
    /// Implementations should ensure that either the previous value remains
    /// intact or the new value is fully committed. Backends that represent a
    /// single file may choose to ignore `key` and always operate on their
    /// configured path.
    ///
    /// # Errors
    /// Returns `PersistenceError::Io` if writing fails.
    fn atomic_write(&self, key: &str, data: &[u8]) -> Result<(), PersistenceError>;
}

/// Loads a snapshot from storage with full integrity verification.
///
/// This replaces the legacy `StorageBackend::load` static method.
///
/// # Errors
/// Returns `PersistenceError` if:
/// - Reading from backend fails.
/// - Data is too short (missing header).
/// - Header validation fails.
/// - CRC checksum mismatch.
pub fn load_snapshot(
    backend: &dyn StorageBackend,
) -> Result<(FileHeader, Vec<u8>), PersistenceError> {
    let data = backend.read()?;

    // 1. Check Buffer Size (Header)
    if data.len() < 64 {
        return Err(PersistenceError::BufferTooSmall {
            expected: 64,
            actual: data.len(),
        });
    }

    // 2. Parse and Validate Header
    let header = FileHeader::from_bytes(&data[0..64])?;

    // 3. Verify Data CRC
    let payload = &data[64..];
    let mut hasher = Hasher::new();
    hasher.update(payload);
    let calculated_crc = hasher.finalize();

    if calculated_crc != header.data_crc {
        return Err(PersistenceError::ChecksumMismatch {
            expected: header.data_crc,
            actual: calculated_crc,
        });
    }

    Ok((header, payload.to_vec()))
}

/// In-memory storage backend for testing and WASM.
#[derive(Debug, Default, Clone)]
pub struct MemoryBackend {
    data: Arc<RwLock<Vec<u8>>>,
}

impl MemoryBackend {
    /// Creates a new empty `MemoryBackend`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

impl StorageBackend for MemoryBackend {
    fn append(&mut self, data: &[u8]) -> Result<(), PersistenceError> {
        let mut storage = self.data.write().map_err(|_| {
            PersistenceError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "lock poisoned",
            ))
        })?;
        storage.extend_from_slice(data);
        Ok(())
    }

    fn read(&self) -> Result<Vec<u8>, PersistenceError> {
        let storage = self.data.read().map_err(|_| {
            PersistenceError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "lock poisoned",
            ))
        })?;
        Ok(storage.clone())
    }

    fn atomic_write(&self, _key: &str, data: &[u8]) -> Result<(), PersistenceError> {
        let mut storage = self.data.write().map_err(|_| {
            PersistenceError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "lock poisoned",
            ))
        })?;
        storage.clear();
        storage.extend_from_slice(data);
        Ok(())
    }
}

#[cfg(not(target_arch = "wasm32"))]
/// File-based storage implementation.
pub mod file {
    use super::{PersistenceError, StorageBackend}; // Minimal imports
                                                   // Re-import things needed from std as they are not in super in a way that * imports cleanly without clippy complaining
                                                   // Actually clippy suggests explicit imports.
    use std::fs::{File, OpenOptions};
    use std::io::{Read, Write};
    use std::path::{Path, PathBuf};
    use std::sync::Mutex;

    /// File-based storage backend.
    #[derive(Debug)]
    pub struct FileBackend {
        path: PathBuf,
        /// Mutex protected file handle to allow interior mutability in atomic_write
        file: Mutex<Option<File>>,
    }

    impl FileBackend {
        /// Creates a new `FileBackend` for the given path.
        pub fn new(path: impl Into<PathBuf>) -> Self {
            Self {
                path: path.into(),
                file: Mutex::new(None),
            }
        }

        fn open_append(&self) -> Result<File, PersistenceError> {
            let mut guard = self.file.lock().map_err(|_| {
                PersistenceError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "lock poisoned",
                ))
            })?;

            if guard.is_none() {
                let file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&self.path)
                    .map_err(PersistenceError::Io)?;
                *guard = Some(file);
            }

            let file = guard.as_ref().ok_or(PersistenceError::NotInitialized)?;
            file.try_clone().map_err(PersistenceError::Io)
        }
    }

    impl StorageBackend for FileBackend {
        fn append(&mut self, data: &[u8]) -> Result<(), PersistenceError> {
            let mut file = self.open_append()?;
            file.write_all(data).map_err(PersistenceError::Io)?;
            file.sync_all().map_err(PersistenceError::Io)?;
            Ok(())
        }

        fn read(&self) -> Result<Vec<u8>, PersistenceError> {
            if !self.path.exists() {
                return Ok(Vec::new());
            }
            let mut file = File::open(&self.path).map_err(PersistenceError::Io)?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)
                .map_err(PersistenceError::Io)?;
            Ok(buffer)
        }

        fn atomic_write(&self, key: &str, data: &[u8]) -> Result<(), PersistenceError> {
            let target = resolve_target_path(&self.path, key);
            let tmp_path = target.with_extension("tmp");
            {
                let mut file = File::create(&tmp_path).map_err(PersistenceError::Io)?;
                file.write_all(data).map_err(PersistenceError::Io)?;
                file.sync_all().map_err(PersistenceError::Io)?;
            }
            std::fs::rename(&tmp_path, &target).map_err(PersistenceError::Io)?;
            sync_parent_dir(&target)?;

            // Invalidate cache
            let mut guard = self.file.lock().map_err(|_| {
                PersistenceError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "lock poisoned",
                ))
            })?;
            *guard = None;

            Ok(())
        }
    }

    fn resolve_target_path(default: &Path, key: &str) -> PathBuf {
        if key.is_empty() {
            return default.to_path_buf();
        }
        let candidate = PathBuf::from(key);
        if candidate.is_absolute() {
            candidate
        } else if let Some(parent) = default.parent() {
            parent.join(candidate)
        } else {
            candidate
        }
    }

    fn sync_parent_dir(path: &Path) -> Result<(), PersistenceError> {
        if let Some(parent) = path.parent() {
            #[cfg(target_os = "windows")]
            {
                use std::os::windows::fs::OpenOptionsExt;
                const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
                // Attempt to open directory for syncing.
                let open_result = OpenOptions::new()
                    .write(true)
                    .custom_flags(FILE_FLAG_BACKUP_SEMANTICS)
                    .open(parent);

                match open_result {
                    Ok(dir) => {
                        // Best effort sync.
                        if let Err(e) = dir.sync_all() {
                            if e.kind() == std::io::ErrorKind::PermissionDenied {
                                return Ok(());
                            }
                            return Err(PersistenceError::Io(e));
                        }
                    }
                    Err(e) => {
                        if e.kind() == std::io::ErrorKind::PermissionDenied {
                            return Ok(());
                        }
                        return Err(PersistenceError::Io(e));
                    }
                }
            }
            #[cfg(not(target_os = "windows"))]
            {
                let dir = File::open(parent).map_err(PersistenceError::Io)?;
                dir.sync_all().map_err(PersistenceError::Io)?;
            }
        }
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
/// IndexedDB storage implementation.
pub mod indexed_db {
    use super::*;
    use wasm_bindgen::prelude::*;
    use web_sys::{IdbDatabase, IdbTransactionMode};

    /// IndexedDB backend.
    #[derive(Debug, Clone)]
    pub struct IndexedDbBackend {
        db: IdbDatabase,
        store_name: String,
        default_key: String,
    }

    impl IndexedDbBackend {
        /// Creates a new `IndexedDbBackend`.
        pub fn new(db: IdbDatabase, store_name: String, default_key: String) -> Self {
            Self {
                db,
                store_name,
                default_key,
            }
        }
    }

    impl StorageBackend for IndexedDbBackend {
        fn append(&mut self, _data: &[u8]) -> Result<(), PersistenceError> {
            Err(PersistenceError::Unsupported(
                "WAL not supported on IDB".to_string(),
            ))
        }

        fn read(&self) -> Result<Vec<u8>, PersistenceError> {
            Err(PersistenceError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Use async_read instead",
            )))
        }

        fn atomic_write(&self, key: &str, data: &[u8]) -> Result<(), PersistenceError> {
            let key = if key.is_empty() {
                &self.default_key
            } else {
                key
            };

            let transaction = self
                .db
                .transaction_with_str_and_mode(&self.store_name, IdbTransactionMode::Readwrite)
                .map_err(|e| {
                    PersistenceError::Io(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("{:?}", e),
                    ))
                })?;

            let store = transaction.object_store(&self.store_name).map_err(|e| {
                PersistenceError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("{:?}", e),
                ))
            })?;

            let js_data = js_sys::Uint8Array::from(data);
            let _request = store
                .put_with_key(&js_data, &JsValue::from_str(key))
                .map_err(|e| {
                    PersistenceError::Io(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("{:?}", e),
                    ))
                })?;

            // Transaction auto-commits when dropped (IdbTransaction::commit() is deprecated)
            // No explicit commit needed - transaction will commit when it goes out of scope
            Ok(())
        }
    }
}
