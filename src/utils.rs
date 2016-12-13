#[macro_escape]
macro_rules! req {
    ($expr:expr) => {
        match $expr {
            Ok(v) => v,
            Err(_why) => return,
        }
    }
}
