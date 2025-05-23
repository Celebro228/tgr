use tgr::{node2d, prelude::*};

struct P;

impl Module for P {
    fn start(&self, _obj: &mut Node2d) {
        //println!("ook!");
        //obj.set_camera();
        //get_text("");
        //
    }

    fn update(&self, obj: &mut Node2d, d: f32) {
        obj.rotation += d as f32 / 10.;
        obj.visible = !obj.visible;
    }

    fn touch(&self, obj: &mut Node2d, _id: u64, touch: &Touch, pos: Vec2) {
        if let Move = touch {
            obj.global_position = vec2(pos.x, pos.y);
        }
    }

    /*fn key(&self, _obj: &mut Node2d, key: &Key, _keymod: miniquad::KeyMods, _touch: &Touch) {
        if let Char(c) = key {
            println!("{:?}", c,)
        }
    }*/
}

struct C;

impl Module for C {
    fn start(&self, _obj: &mut Node2d) {
        //get_data::<Audio>("ok").unwrap().play();
        //get_stat(0);
        //set_camera(100., 0.);
        //set_window(get_window().x / 2., get_window().y / 2.);
        //wait(100.);
    }

    fn update(&self, obj: &mut Node2d, d: f32) {
        let p = 100. * d;
        //println!("{}", get_fps());
        //println!("{} {}", get_mouse().x, get_mouse().y)
        //set_canvas(get_canvas().x + p, get_canvas().y);
        let mut cam = Camera2d.get();
        cam.x += p;
        Camera2d.set(cam);

        //let num= get_data::<u8>("num").unwrap();

        add_stat(0, 1.);
        //save_stat(0, *get_stat(0));

        //println!("{}", get_stat(0));

        let a = format!("{}", get_stat(0));

        obj.obj.set_text(&a);

        //println!("{}", get_fps())
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
    //set_data("ok", audio("./test/test.ogg"));

    set_data("a", 0u8);

    set_stat(0, 0.); //load_stat(0));

    let font = font("test/calibri.ttf");

    let o = String::from("ok");
    let s = node2d![
        //image("img", &texture("./image_example.png")),
        rect("rect", 500., 500., 100.)
            .scale(2., 2.)
            .position(100., 100.)
            .rotation(-180.)
            .color(rgb(255, 0, 0))
            .keep(Center)
            .node(vec![rect("rect", 100., 100., 0.)
                .position(200., 0.)
                .visible(true)
                .color(rgb(170, 255, 0))
                .node(vec![circle(&o, 200.)
                    .position(0., -200.)
                    .script(&P)
                    .color(hsv(50., 50., 50.))
                    .color(rgba(209, 30, 30, 0.74))
                    .rotation(100.)
                    .scale(2., 1.)
                    .offset(1., 0.)])]),
        text("kok", "Привет, пупсик", 250., &font)
            //.position(1000., 0.)
            .rotation(0.)
            .keep(Left)
            .script(&C)
            .offset(1., 0.),
        //button("ok", "asasdasdd", 500., &font),
        //check("a", 500.)
        //edittext("tok", "hudden", 250., &font).offset(1., 0.), //scroll()
        line("asd", 0., 0., 500., 500., 100., 10.),
        //joystick
        //button
        //text
        //label
        //image("img", "./test/python.png")
        //text("ok", "as", 500., "./text")
    ]
    .script(&SCENE);

    Camera2d.view(KeepHeight, KeepWidth).set_zoom(2.);

    Engine
        .node2d(s)
        .fullscreen(false)
        .window(1280., 720.)
        .canvas(1000., 1000.)
        .backgraund(rgba(0, 0, 0, 0.2))
        //.canvas(1000, 1000)
        //.camera(100., 0.)
        //.mouse(KEEP_IN)
        //.touch_in_mouse(false)
        //.backgraund()
        //.font()
        //.data()
        .start("Tgr test");
}
