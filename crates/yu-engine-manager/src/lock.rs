use crate::{EngineManager, ManagerError, validate_identifier};
use std::fs::{File, OpenOptions, TryLockError};

#[derive(Debug)]
pub struct EngineMutationLock {
    file: File,
}

impl Drop for EngineMutationLock {
    fn drop(&mut self) {
        let _ = self.file.unlock();
    }
}

impl EngineManager {
    pub(crate) fn acquire_engine_lock(
        &self,
        engine_id: &str,
    ) -> Result<EngineMutationLock, ManagerError> {
        validate_identifier("engine id", engine_id)?;

        let lock_dir = self.layout().state_dir().join("locks");
        std::fs::create_dir_all(&lock_dir).map_err(|error| {
            ManagerError::Io(format!(
                "cannot create engine lock directory {}: {error}",
                lock_dir.display()
            ))
        })?;

        let path = lock_dir.join(format!("{engine_id}.lock"));
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path)
            .map_err(|error| {
                ManagerError::Io(format!(
                    "cannot open engine mutation lock {}: {error}",
                    path.display()
                ))
            })?;

        match file.try_lock() {
            Ok(()) => Ok(EngineMutationLock { file }),
            Err(TryLockError::WouldBlock) => Err(ManagerError::Busy(format!(
                "engine {engine_id} is being modified by another YuTool process"
            ))),
            Err(TryLockError::Error(error)) => Err(ManagerError::Io(format!(
                "cannot lock engine {engine_id}: {error}"
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EngineTarget, ManagedLayout};
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn temp_root(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after epoch")
            .as_nanos();

        std::env::temp_dir().join(format!(
            "yu-engine-lock-{label}-{}-{nonce}",
            std::process::id()
        ))
    }

    fn manager(root: &std::path::Path) -> EngineManager {
        EngineManager::new(
            ManagedLayout::new(root),
            EngineTarget::new("test-os", "test-arch"),
        )
    }

    #[test]
    fn second_process_style_lock_attempt_is_busy() {
        let root = temp_root("busy");
        let manager = manager(&root);

        let first = manager.acquire_engine_lock("fixture-engine").unwrap();
        let second = manager.acquire_engine_lock("fixture-engine");

        assert!(matches!(second, Err(ManagerError::Busy(_))));

        drop(first);
        assert!(manager.acquire_engine_lock("fixture-engine").is_ok());

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn different_engines_can_be_locked_independently() {
        let root = temp_root("independent");
        let manager = manager(&root);

        let first = manager.acquire_engine_lock("engine-a").unwrap();
        let second = manager.acquire_engine_lock("engine-b").unwrap();

        drop(first);
        drop(second);
        let _ = fs::remove_dir_all(root);
    }
}
