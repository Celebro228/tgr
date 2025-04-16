use tgr::{engine::*, node2d};

struct P;

impl Module for P {
    fn start(&self, _obj: &mut Node2d) {
        //println!("ook!");
        //obj.set_camera();
    }

    fn touch(&self, obj: &mut Node2d, _id: u64, touch: &Touch, pos: Vec2) {
        if let Move = touch {
            obj.set_global_position(pos.x, pos.y);
        }
    }
}

struct C;

impl Module for C {
    fn start(&self, _obj: &mut Node2d) {
        //set_camera(100., 0.);
        //set_window(get_window().x / 2., get_window().y / 2.);
        //wait(100.);
    }

    fn update(&self, _obj: &mut Node2d, d: f64) {
        let p = 100. * d as f32;
        println!("{}", get_fps());
        //println!("{} {}", get_mouse().x, get_mouse().y)
        //set_canvas(get_canvas().x + p, get_canvas().y);
        set_camera(get_camera().x + p, get_camera().y);
        //set_window(100., 100.);
        //obj.position.x += 1.;
    }
}

struct SCENE;

impl Module for SCENE {
    fn start(&self, _obj: &mut Node2d) {
        //let p = get_canvas();
        /*for i in 0..10000 {
            obj.add_node(vec![rect(&format!("{}", i), 500., 500.)]);
        }*/
        //let m = get_window_position();
    }
}

fn main() {

    let o = String::from("ok");
    let s = node2d![
        //image("img", &texture("./image_example.png")),
        rect("rect", 500., 500., 100.)
            .scale(2., 2.)
            .position(100., 100.)
            .color(rgb(255, 0, 0))
            .script(&C)
            .keep(Center)
            .node(vec![rect("rect", 100., 100., 0.)
                .position(200., 0.)
                .visible(true)
                .color(rgb(170, 255, 0))
                .node(vec![circle(&o, 200.)
                    .position(0., -200.)
                    .script(&P)
                    .color(hsv(50., 50., 50.))
                    .color(rgba(255, 255, 255, 100))
                    .scale(1., 1.)])]),
        image("img", &text("Привет, пупсик", 500., &font("./test/calibri.ttf")))
            .position(300., 0.),
        //
        //image("img", "./test/python.png")
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
        //.backgraund()
        //.font()
        .start("Title");
}
