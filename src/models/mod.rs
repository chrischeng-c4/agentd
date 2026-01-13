pub mod challenge;
pub mod change;
pub mod requirement;
pub mod scenario;
pub mod verification;

pub use challenge::{Challenge, ChallengeIssue, ChallengeVerdict, IssueSeverity};
pub use change::{Change, ChangePhase, SpecterConfig};
pub use requirement::{Requirement, RequirementDelta};
pub use scenario::Scenario;
pub use verification::{TestResult, TestStatus, Verification};
