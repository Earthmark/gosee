use std::collections::BTreeMap;
use std::io::ErrorKind;
use std::path::Path;
use tokio::fs;
use tokio::sync::{RwLock, RwLockReadGuard};

pub struct NameStore {
    path: Box<Path>,
    backup_path: Box<Path>,
    records: RwLock<BTreeMap<String, String>>,
}

impl NameStore {
    pub async fn new(path: &Path, backup_path: &Path) -> Result<Self, std::io::Error> {
        let obj = Self {
            path: path.into(),
            backup_path: backup_path.into(),
            records: Default::default(),
        };
        obj.read_disk().await?;
        Ok(obj)
    }

    pub async fn get(&self, name: &str) -> Option<String> {
        self.records.read().await.get(name).cloned()
    }
    pub async fn set(&self, name: &str, value: &str) -> Result<(), std::io::Error> {
        let data = self.update_self(name, value).await;

        self.write_disk(&data).await?;

        Ok(())
    }

    async fn update_self(
        &self,
        name: &str,
        value: &str,
    ) -> RwLockReadGuard<BTreeMap<String, String>> {
        let mut store = self.records.write().await;
        store.entry(name.into()).or_insert(value.into());
        store.downgrade()
    }

    async fn write_disk(&self, map: &BTreeMap<String, String>) -> Result<(), std::io::Error> {
        let data = serde_json::ser::to_vec_pretty(map).unwrap();
        fs::write(&self.backup_path, data).await?;
        fs::copy(&self.backup_path, &self.path).await?;
        Ok(())
    }

    async fn read_disk(&self) -> Result<(), std::io::Error> {
        let mut store = self.records.write().await;

        let data = match fs::read(&self.path).await {
          Ok(data) => data,
          Err(e) if e.kind() == ErrorKind::NotFound => "{}".as_bytes().to_vec(),
          _ => fs::read(&self.backup_path).await?
        };

        let data = serde_json::de::from_slice(&data)?;

        store.clear();
        *store = data;
        Ok(())
    }
}
