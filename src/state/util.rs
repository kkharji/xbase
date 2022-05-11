macro_rules! castable {
    ($name:expr, $t:ident) => {
        impl Entry {
            paste::paste! {
                    pub fn [<as _ $name>](&self) -> Option<&$t> {
                        if let Value::$t(v) = &self.value {
                            Some(v)
                        } else {
                            None
                        }
                    }

                #[must_use]
                pub fn [<is _ $name>](&self) -> bool {
                    matches!(self.value, Value::$t(..))
                }
            }
        }
    };
}
pub(super) use castable;
