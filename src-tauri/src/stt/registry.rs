use std::{collections::HashMap, sync::Arc};

use crate::{
    error::{VoxError, VoxResult},
    ports::{EngineId, SttEngine},
};

#[derive(Default)]
pub struct EngineRegistry {
    engines: HashMap<EngineId, Arc<dyn SttEngine>>,
}

impl EngineRegistry {
    pub fn register(&mut self, engine: Arc<dyn SttEngine>) -> VoxResult<()> {
        let id = engine.id();
        if self.engines.insert(id.clone(), engine).is_some() {
            return Err(VoxError::Stt(format!(
                "engine '{}' is already registered",
                id.0
            )));
        }
        Ok(())
    }

    pub fn get(&self, id: &str) -> VoxResult<Arc<dyn SttEngine>> {
        self.engines
            .get(&EngineId::new(id))
            .cloned()
            .ok_or_else(|| VoxError::Stt(format!("unknown transcription engine '{id}'")))
    }

    pub fn list(&self) -> Vec<Arc<dyn SttEngine>> {
        let mut engines = self.engines.values().cloned().collect::<Vec<_>>();
        engines.sort_by_key(|engine| engine.id().0);
        engines
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use async_trait::async_trait;

    use super::*;
    use crate::ports::{AudioClip, EngineCaps, LanguageHint, Transcript};

    struct FakeEngine(&'static str);

    #[async_trait]
    impl SttEngine for FakeEngine {
        fn id(&self) -> EngineId {
            EngineId::new(self.0)
        }

        fn capabilities(&self) -> EngineCaps {
            EngineCaps {
                languages: vec!["en".to_owned()],
                multilingual: false,
                streaming: false,
                model_size_bytes: 0,
            }
        }

        async fn load(&self, _model_path: &Path) -> VoxResult<()> {
            Ok(())
        }

        async fn transcribe(
            &self,
            _audio: AudioClip,
            _language: LanguageHint,
        ) -> VoxResult<Transcript> {
            Ok(Transcript {
                text: "test".to_owned(),
                language: "en".to_owned(),
                segments: vec![],
            })
        }
    }

    #[test]
    fn registry_rejects_duplicate_engine_ids() {
        let mut registry = EngineRegistry::default();
        registry.register(Arc::new(FakeEngine("fake"))).unwrap();
        assert!(registry.register(Arc::new(FakeEngine("fake"))).is_err());
    }

    #[test]
    fn registry_lists_engines_deterministically() {
        let mut registry = EngineRegistry::default();
        registry.register(Arc::new(FakeEngine("zeta"))).unwrap();
        registry.register(Arc::new(FakeEngine("alpha"))).unwrap();
        let ids = registry
            .list()
            .iter()
            .map(|engine| engine.id().0)
            .collect::<Vec<_>>();
        assert_eq!(ids, vec!["alpha", "zeta"]);
    }
}
