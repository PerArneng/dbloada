pub trait Logger {
    fn error(&self, msg: &str);
    fn warn(&self, msg: &str);
    fn info(&self, msg: &str);
    fn debug(&self, msg: &str);
    fn trace(&self, msg: &str);
}
