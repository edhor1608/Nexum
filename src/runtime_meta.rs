use std::collections::BTreeMap;

use crate::capsule::Capsule;

pub fn capsule_runtime_env(capsule: &Capsule) -> BTreeMap<String, String> {
    let mut env = BTreeMap::new();
    env.insert("NEXUM_CAPSULE_ID".to_string(), capsule.capsule_id.clone());
    env.insert("NEXUM_CAPSULE_SLUG".to_string(), capsule.slug.clone());
    env.insert("NEXUM_CAPSULE_DOMAIN".to_string(), capsule.domain());
    env.insert(
        "NEXUM_CAPSULE_WORKSPACE".to_string(),
        capsule.workspace.to_string(),
    );
    env
}

pub fn terminal_process_label(capsule_id: &str) -> String {
    format!("nexum-terminal-{capsule_id}")
}
