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
Разделить создание объекта от самого объекта!
Добавить кэш для отрисовки 2д объекта!
Оптимизировать рендер-лист
Оптимизировать текст под маленький холст?
Изменить код камеры?
*/
