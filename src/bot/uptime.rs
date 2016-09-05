// ISC License (ISC)
//
// Copyright (c) 2016, Austin Hellyer <hello@austinhellyer.me>
//
// Permission to use, copy, modify, and/or distribute this software for any
// purpose with or without fee is hereby granted, provided that the above
// copyright notice and this permission notice appear in all copies.
//
// THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH
// REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY
// AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT,
// INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM
// LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR
// OTHER TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR
// PERFORMANCE OF THIS SOFTWARE.

use chrono::{DateTime, UTC};
use std::default::Default;

pub struct Uptime {
    /// Unix timestamp of when the program itself started
    pub boot: DateTime<UTC>,
    /// Unix timestamp of when the current connection was made. This should
    /// probably _technically_ be an Option, _but_ a user will never be able to
    /// request the uptime if there is no connection, so it's okay.
    pub connection: DateTime<UTC>,
}

impl Default for Uptime {
    fn default() -> Uptime {
        Uptime {
            boot: UTC::now(),
            connection: UTC::now(),
        }
    }
}
