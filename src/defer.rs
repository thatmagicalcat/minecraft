#[macro_export]
macro_rules! defer {
    [ $($tt:tt)* ] => {
        let __defer = $crate::defer::Defer(Some(Box::new(|| { $($tt)* } )));
    };
}

pub struct Defer<F: FnOnce()>(pub Option<Box<F>>);
impl<F: FnOnce()> Drop for Defer<F> {
    fn drop(&mut self) {
        self.0.take().unwrap()();
    }
}
