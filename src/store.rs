use rkv::{
    backend::{SafeMode, SafeModeDatabase, SafeModeEnvironment},
    Manager, Rkv, SingleStore, StoreOptions, Value,
};
use std::{path::Path, sync::Arc, sync::RwLock};

pub struct NameStore {
    env: Arc<RwLock<Rkv<SafeModeEnvironment>>>,
    db: SingleStore<SafeModeDatabase>,
}

type Result<T> = std::result::Result<T, rkv::StoreError>;
pub type Error = rkv::StoreError;

impl NameStore {
    pub fn new(path: &Path) -> Result<Self> {
        std::fs::create_dir_all(path)?;
        let mut manager = Manager::<SafeModeEnvironment>::singleton().write()?;
        let env = manager.get_or_create(path, Rkv::new::<SafeMode>)?;

        let db = {
            let env_handle = env.read()?;
            env_handle.open_single("links", StoreOptions::create())?
        };

        Ok(Self { db, env })
    }

    pub fn get(&self, name: &str) -> Result<Option<String>> {
        let env = self.env.read()?;
        let reader = env.read()?;
        let result = if let Some(Value::Str(s)) = self.db.get(&reader, name)? {
            Some(s.into())
        } else {
            None
        };
        Ok(result)
    }

    pub fn set(&self, name: &str, value: &str) -> Result<()> {
        let env = self.env.read()?;
        let mut writer = env.write()?;
        self.db.put(&mut writer, name, &Value::Str(value))?;
        writer.commit()?;
        Ok(())
    }
}
