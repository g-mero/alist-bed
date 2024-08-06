use std::sync::LazyLock;

use quick_cache::sync::Cache;

static GLOBAL_CACHE: LazyLock<Cache<String, Vec<u8>>> = LazyLock::new(|| {
    Cache::new(100)
});

pub fn get(key: &str) -> Option<Vec<u8>> {
    GLOBAL_CACHE.get(key)
}

pub fn clear() {
    GLOBAL_CACHE.clear()
}

pub fn set(key: String, value: Vec<u8>) {
    GLOBAL_CACHE.insert(key, value);
}