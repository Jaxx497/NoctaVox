use std::sync::atomic::{AtomicU8, AtomicUsize, Ordering::Relaxed};

pub enum RefreshStage {
    Scanning = 0,
    Parsing = 1,
    Database = 2,
    Rebuilding = 3,
}

impl RefreshStage {
    pub fn label(self) -> &'static str {
        match self {
            RefreshStage::Scanning => "Scanning songs...",
            RefreshStage::Parsing => "Processing",
            RefreshStage::Database => "Updating database...",
            RefreshStage::Rebuilding => "Rebuilding library...",
        }
    }

    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => RefreshStage::Scanning,
            1 => RefreshStage::Parsing,
            2 => RefreshStage::Database,
            _ => RefreshStage::Rebuilding,
        }
    }
}

#[derive(Default)]
pub struct RefreshProgress {
    percent: AtomicU8,
    stage: AtomicU8,
    current: AtomicUsize,
    total: AtomicUsize,
}

impl RefreshProgress {
    pub fn set(&self, stage: RefreshStage, percent: u8) {
        self.stage.store(stage as u8, Relaxed);
        self.percent.store(percent, Relaxed);
    }

    pub fn set_counts(&self, current: usize, total: usize) {
        self.current.store(current, Relaxed);
        self.total.store(total, Relaxed);
    }

    pub fn percent(&self) -> u8 {
        self.percent.load(Relaxed)
    }

    pub fn stage(&self) -> RefreshStage {
        RefreshStage::from_u8(self.stage.load(Relaxed))
    }

    pub fn counts(&self) -> (usize, usize) {
        (self.current.load(Relaxed), self.total.load(Relaxed))
    }
}
