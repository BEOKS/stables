/// Runtime plan exposed for user visibility and orchestration tracing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    /// Complete ordered plan entries. Updates replace the whole list.
    pub entries: Vec<PlanEntry>,
}

/// One visible plan step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanEntry {
    /// Human-readable task description.
    pub content: String,
    /// Relative importance of this plan entry.
    pub priority: PlanEntryPriority,
    /// Current lifecycle state.
    pub status: PlanEntryStatus,
}

/// Relative importance of a plan entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanEntryPriority {
    /// Critical path item.
    High,
    /// Important but not blocking.
    Medium,
    /// Nice-to-have or cleanup item.
    Low,
}

/// Current lifecycle state of a plan entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanEntryStatus {
    /// Work has not started.
    Pending,
    /// Work is currently active.
    InProgress,
    /// Work is complete.
    Completed,
}
