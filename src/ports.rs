use std::collections::{BTreeSet, HashMap};

#[derive(Debug, Clone)]
pub struct PortAllocator {
    start: u16,
    end: u16,
    used: BTreeSet<u16>,
    by_capsule: HashMap<String, u16>,
}

impl PortAllocator {
    pub fn new(start: u16, end: u16) -> Self {
        assert!(start <= end, "port range start must be <= end");
        Self {
            start,
            end,
            used: BTreeSet::new(),
            by_capsule: HashMap::new(),
        }
    }

    pub fn allocate(&mut self, capsule_id: &str) -> Option<u16> {
        if let Some(existing) = self.by_capsule.get(capsule_id) {
            return Some(*existing);
        }

        for candidate in self.start..=self.end {
            if self.used.insert(candidate) {
                self.by_capsule.insert(capsule_id.to_string(), candidate);
                return Some(candidate);
            }
        }

        None
    }

    pub fn reserve(&mut self, port: u16) {
        assert!(
            (self.start..=self.end).contains(&port),
            "reserved port out of range"
        );
        self.used.insert(port);
    }

    pub fn release(&mut self, capsule_id: &str) {
        if let Some(port) = self.by_capsule.remove(capsule_id) {
            self.used.remove(&port);
        }
    }

    pub fn used_ports(&self) -> &BTreeSet<u16> {
        &self.used
    }

    pub fn range(&self) -> (u16, u16) {
        (self.start, self.end)
    }
}
