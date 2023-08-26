#[macro_export]
macro_rules! init {
    ($dir:expr) => {
        $crate::load_translations(
            std::path::Path::new($dir)
        ).expect("Cannot load translations")
    };
}

#[macro_export]
macro_rules! message {
    ($lang:expr,$path:expr) => {
        $crate::fmt::translate($lang, $path, &$crate::fmt::formatter::Formatter::new())
    };

    ($lang:expr,$path:expr,$formatter:expr) => {
        $crate::fmt::translate($lang, $path, &$formatter)
    };
}