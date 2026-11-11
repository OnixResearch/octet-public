//! Dependency of the `macro_dependency_custom` UI fixture: its exported
//! macro defines a `macro_rules!` helper in the invoking crate whose body
//! holds governed allows. Those allows are this crate's, not the invoker's.

#[macro_export]
macro_rules! define_helper {
    () => {
        macro_rules! helper {
            () => {
                #[allow(tigerstyle::structure)]
                mod helped {}

                #[allow(tigerstyle::no_unwrap)]
                mod unwrapping {}
            };
        }
    };
}
