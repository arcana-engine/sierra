pub use crate::backend::Queue;
use {
    crate::OutOfMemory,
    std::{error::Error, fmt::Debug},
};

/// Capability a queue may have.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub enum Capability {
    Transfer,
    Compute,
    Graphics,
}

bitflags::bitflags! {
    /// Queue capability flags.
    #[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
    pub struct QueueCapabilityFlags: u32 {
        const TRANSFER  = 0b001;
        const COMPUTE   = 0b010;
        const GRAPHICS  = 0b100;
    }
}

impl QueueCapabilityFlags {
    /// Check if queue with those flags supports specified capability.
    pub fn supports(&self, other: Capability) -> bool {
        match other {
            Capability::Transfer => self.contains(Self::TRANSFER),
            Capability::Compute => self.contains(Self::COMPUTE),
            Capability::Graphics => self.contains(Self::GRAPHICS),
        }
    }

    /// Check if queue with those flags supports specified capability.
    pub fn supports_graphics(&self) -> bool {
        self.contains(Self::GRAPHICS)
    }

    /// Check if queue with those flags supports specified capability.
    pub fn supports_compute(&self) -> bool {
        self.contains(Self::COMPUTE)
    }
}

/// Information about one queue family.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct FamilyInfo {
    /// Supported capabilities.
    /// All queues of one family have same set of capabilities.
    pub capabilities: QueueCapabilityFlags,

    /// Maximum number of queues from this family that can be created.
    pub count: usize,
}

/// Family of queues created togther with device.
#[derive(Debug)]
pub struct Family {
    pub capabilities: QueueCapabilityFlags,
    pub queues: Vec<Queue>,
}

impl Family {
    pub fn supports(&self, capability: Capability) -> bool {
        self.capabilities.supports(capability)
    }

    pub fn take(&mut self, count: usize) -> impl Iterator<Item = Queue> + '_ {
        std::iter::from_fn(move || self.queues.pop()).take(count)
    }
}

/// Trait for querying command queues.
pub trait QueuesQuery {
    type Error: Error + 'static;
    type Queues;
    type Query: AsRef<[(usize, usize)]>;
    type Collector;

    fn query(
        self,
        families: &[FamilyInfo],
    ) -> Result<(Self::Query, Self::Collector), Self::Error>;

    fn collect(
        collector: Self::Collector,
        families: Vec<Family>,
    ) -> Self::Queues;
}

#[derive(Clone, Copy, Debug)]
pub struct QueuesQueryClosure<F>(pub F);

impl<F, Q, E> QueuesQuery for QueuesQueryClosure<F>
where
    F: FnOnce(&[FamilyInfo]) -> Result<Q, E>,
    Q: IntoIterator<Item = (usize, usize)>,
    E: Error + 'static,
{
    type Collector = ();
    type Error = E;
    type Query = Vec<(usize, usize)>;
    type Queues = Vec<Family>;

    fn query(
        self,
        families: &[FamilyInfo],
    ) -> Result<(Self::Query, Self::Collector), E> {
        Ok(((self.0)(families)?.into_iter().collect(), ()))
    }

    fn collect(_collector: (), families: Vec<Family>) -> Self::Queues {
        families
    }
}

/// Query only one queue with specified capabilities.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct SingleQueueQuery(QueueCapabilityFlags);

impl SingleQueueQuery {
    pub const COMPUTE: Self = SingleQueueQuery(QueueCapabilityFlags::COMPUTE);
    pub const GENERAL: Self =
        SingleQueueQuery(QueueCapabilityFlags::from_bits_truncate(0b11));
    pub const GRAPHICS: Self = SingleQueueQuery(QueueCapabilityFlags::GRAPHICS);
    pub const TRANSFER: Self = SingleQueueQuery(QueueCapabilityFlags::TRANSFER);
}

/// Could not find a queue with specified capabilities.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct QueueNotFound(QueueCapabilityFlags);

impl std::fmt::Display for QueueNotFound {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            fmt,
            "Could not find a queue with following capabilities: {:?}",
            self.0,
        )
    }
}

impl std::error::Error for QueueNotFound {}

impl QueuesQuery for SingleQueueQuery {
    type Collector = ();
    type Error = QueueNotFound;
    type Query = [(usize, usize); 1];
    type Queues = Queue;

    fn query(
        self,
        families: &[FamilyInfo],
    ) -> Result<([(usize, usize); 1], ()), QueueNotFound> {
        for (index, family) in families.iter().enumerate() {
            if family.count > 0 && family.capabilities.contains(self.0) {
                return Ok(([(index, 1)], ()));
            }
        }

        Err(QueueNotFound(self.0))
    }

    fn collect(_collector: (), mut families: Vec<Family>) -> Queue {
        assert_eq!(families.len(), 1);
        assert_eq!(families[0].queues.len(), 1);
        families.remove(0).queues.remove(0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct QueueId {
    pub family: usize,
    pub index: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum PresentError {
    #[error(transparent)]
    OutOfMemory {
        #[from]
        source: OutOfMemory,
    },
    #[error("Surface is out of date")]
    OutOfDate,

    #[error("Surface was lost")]
    SurfaceLost,
    // FullScreenExclusiveModeLost,
}

pub enum PresentOk {
    Success,
    Suboptimal,
}
