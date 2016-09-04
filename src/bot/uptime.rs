use chrono::{DateTime, UTC};

pub struct Uptime {
    /// Unix timestamp of when the program itself started
    pub boot: DateTime<UTC>,
    /// Unix timestamp of when the current connection was made. This should
    /// probably _technically_ be an Option, _but_ a user will never be able to
    /// request the uptime if there is no connection, so it's okay.
    pub connection: DateTime<UTC>,
}
