use tgr::{engine::*, node2d};

struct P;

impl Module for P {
    fn start(&self, obj: &mut Node2d) {
        println!("ook!");
        //obj.set_camera();
    }

    fn touch(&self, obj: &mut Node2d, id: u64, touch: &Touch, pos: Vec2) {
        if let Move = touch {
            obj.set_global_position(pos.x, pos.y);
        }
    }
}

struct C;

impl Module for C {
    fn start(&self, obj: &mut Node2d) {
        //set_window(get_window().x / 2., get_window().y / 2.);
        //wait(100.);
    }

    fn update(&self, obj: &mut Node2d, d: f64) {
        let p = 100. * d as f32;
        println!("{}", 1. / d)
        //println!("{} {}", get_mouse().x, get_mouse().y)
        //set_canvas(get_canvas().x + p, get_canvas().y);
        //set_camera(get_camera().x + p, get_camera().y);
        //set_window(100., 100.);
        //obj.position.x += 1.;
    }
}

struct SCENE;

impl Module for SCENE {
    fn start(&self, obj: &mut Node2d) {
        let p = get_canvas();
        for i in 0..50000 {
            obj.add_node(vec![rect(&format!("{}", i), 500., 500.)]);
        }
        //set_canvas(p / 2.);
        //let m = get_window_position();
        //let mut s = obj.get_node("rect").unwrap();
        //s.position.y = 1000.;
    }
}

fn main() {
    let o = String::from("ok");
    let mut s = node2d![
        rect("rect", 500., 500.)
            .scale(2., 2.)
            .position(100., 100.)
            .color(rgb(247, 0, 0))
            .script(&C)
            .node(vec![rect("rect", 100., 100.)
                .position(200., 0.)
                .visible(true)
                .color(rgb(170, 255, 0))
                .node(vec![circle(&o, 100.)
                    .position(0., -200.)
                    .script(&P)
                    .color(rgb(51, 0, 255))
                    .scale(3., 1.)])]),
        //image("img", "./data_store")
        //text("ok", "Heeloo!!!1", "./text")
    ]
    .script(&SCENE);

    Engine
        .node2d(s)
        .fullscreen(false)
        .window(1280., 720.)
        .canvas(1000., 1000.)
        //.canvas(1000, 1000)
        .view(KeepHeight, KeepWidth)
        //.camera(100., 0.)
        .zoom(2.)
        //.mouse(KEEP_IN)
        //.touch_in_mouse(false)
        .start("Title");
}
