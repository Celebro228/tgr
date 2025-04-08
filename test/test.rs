use tgr::{engine::*, node2d};

struct P;

impl Module for P {
    fn start(&self, obj: &mut Node2d) {
        println!("ook!");
    }
}

struct C;

impl Module for C {
    fn start(&self, obj: &mut Node2d) {
        //set_window(get_window().x / 2., get_window().y / 2.);
    }

    fn update(&self, obj: &mut Node2d, d: f64) {
        //obj.position.x += 1.;
    }
}

struct SCENE;

impl Module for SCENE {
    fn start(&self, obj: &mut Node2d) {
        let p = get_canvas();
        //set_canvas(p / 2.);
        //let m = get_window_position();
        let mut s = obj.get_node("rect").unwrap();
        s.position.y = 10.;
    }
}

fn main() {
    let o = String::from("ok");
    let mut s = node2d![rect("rect", 100., 100.)
        .position(100., 100.)
        .script(&C)
        .node(vec![rect("rect", 100., 100.)
            .position(200., 0.)
            //visible(false)
            .node(vec![circle(&o, 50.).position(0., -200.).script(&P)])]),];

    Engine
        .script(&P)
        .node2d(s)
        .fullscreen(false)
        .window(1280., 720.)
        //.canvas(1000, 1000)
        //.view(Scale, Out)
        //.mouse(View)
        .start("Title");
}
