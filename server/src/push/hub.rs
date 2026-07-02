use std::collections::HashMap;
use std::sync::Arc;

use crate::push::PushProvider;

pub struct PushHub {
    providers: HashMap<String, Arc<dyn PushProvider>>,
}

impl PushHub {
    pub fn new(providers: Vec<Arc<dyn PushProvider>>) -> Self {
        let mut map = HashMap::new();
        for provider in providers {
            map.insert(provider.platform().to_string(), provider);
        }
        Self { providers: map }
    }

    pub fn get(&self, platform: &str) -> Option<Arc<dyn PushProvider>> {
        self.providers.get(platform).cloned()
    }

    pub fn platforms(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }
}
