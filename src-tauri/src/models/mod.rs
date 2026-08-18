pub mod audit;
pub mod bed;
pub mod encounter;
pub mod patient;
pub mod session;
pub mod user;

pub use audit::AuditLogEntry;
pub use bed::{
    ActiveAssignmentRef, Bed, BedAssignment, BedSummary, Floor, FloorLayout, Room, RoomStatus,
};
pub use encounter::{Diagnosis, Encounter, Evolution, Treatment};
pub use patient::Patient;
pub use session::Session;
pub use user::User;
