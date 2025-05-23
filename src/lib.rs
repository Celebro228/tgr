// Engine
pub mod data;
pub mod engine;
pub mod object;
pub mod physic;
pub mod prelude;
pub mod render;

// Option
#[cfg(feature = "audio")]
pub mod audio;

#[cfg(feature = "widgets")]
pub mod widgets;

pub mod info;

/*
Оптимизировать листы
Оптимизировать текст под маленький холст?
*/
