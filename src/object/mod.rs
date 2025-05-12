pub mod d2;

pub enum Keep {
    Canvas,
    Center,
    Up,
    Down,
    Left,
    Right,
    LeftUp,
    LeftDown,
    RightUp,
    RightDown,
}

pub enum Touch {
    Press,
    Relese,
    Move,
}

pub enum Key {
    Char(char),
    //Code(KeyCode),
}

/*use inventory;

pub use tgr_macro::module;

inventory::collect!(&'static dyn Module);

pub fn start_all() {
    for module in inventory::iter::<&'static dyn Module> {
        module.start();
    }
}*/
