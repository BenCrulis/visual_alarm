use std::os::raw::c_int;

use x11rb::connect;
use x11rb::rust_connection::RustConnection;
use x11rb::connection::{RequestConnection, Connection};
use x11rb::protocol::xproto::{get_window_attributes, Screen, VisualClass, ColormapAlloc, create_colormap, CreateWindowAux, EventMask, WindowClass, ClipOrdering, CreateGCAux, create_gc, map_window, ConnectionExt};
use x11rb::protocol::xproto::create_window as  cw;
use x11rb::protocol::shape::{SO, SK};
use x11rb::protocol::shape::ConnectionExt as _;

use cairo;

use crate::util::find_visual_with_depth;

pub struct Display {
    conn: RustConnection,
    win_id: u32,
    width: u16,
    height: u16,
    cairo_context: cairo::Context
}


impl Display {
    pub fn create_and_connect() -> Result<Self, Box<dyn std::error::Error>> {
        let (conn, screen_num) = connect(None)?;

        conn.extension_information(x11rb::protocol::xfixes::X11_EXTENSION_NAME)
            .expect("failed to get extension information")
            .expect("xfixes extension not present");

        let setup = conn.setup();

        let screen: &Screen = &setup.roots[screen_num];

        let root = screen.root;

        let width = screen.width_in_pixels;
        let height = screen.height_in_pixels;

        let vinfo = get_window_attributes(&conn, root).unwrap().reply().unwrap();

        let win_id = conn.generate_id().unwrap();
        let gc_id = conn.generate_id().unwrap();
        let cm_id = conn.generate_id().unwrap();

        let win_depth = 32;
        let visual_ob = find_visual_with_depth(&screen.allowed_depths, win_depth, VisualClass::TRUE_COLOR).unwrap();
        let visual_id = visual_ob.visual_id;

        let mut vvisual = x11::xlib::Visual{
            ext_data: std::ptr::null_mut(),
            visualid: 0,
            class: 0,
            red_mask: 0,
            green_mask: 0,
            blue_mask: 0,
            bits_per_rgb: 0,
            map_entries: 0
        };

        let (mut x11_visual, mut dis) = unsafe {
            let dis = x11::xlib::XOpenDisplay(std::ptr::null());

            let mut visual_info: x11::xlib::XVisualInfo = x11::xlib::XVisualInfo{
                visual: &mut vvisual,
                visualid: 0,
                screen: 0,
                depth: 0,
                class: 0,
                red_mask: 0,
                green_mask: 0,
                blue_mask: 0,
                colormap_size: 0,
                bits_per_rgb: 0
            };

            let screen_id = x11::xlib::XDefaultScreen(dis);
            x11::xlib::XMatchVisualInfo(dis, screen_id, win_depth as i32, x11::xlib::TrueColor, &mut visual_info);
            (visual_info, dis)
        };


        let visual_id = x11_visual.visualid as u32;


        create_colormap(&conn,ColormapAlloc::NONE,cm_id,root, visual_id).unwrap().check().unwrap();

        let mut win_aux = CreateWindowAux::new();
        win_aux.background_pixel = Some(0);
        win_aux.override_redirect = Some(1);
        win_aux.border_pixel = Some(1);
        win_aux.event_mask = Some(0);
        win_aux.colormap = Some(cm_id);
        win_aux.background_pixmap = None;

        let captured_events: u32 = (EventMask::POINTER_MOTION | EventMask::POINTER_MOTION_HINT | EventMask::BUTTON_PRESS
            | EventMask::KEY_PRESS
            | EventMask::KEY_RELEASE | EventMask::STRUCTURE_NOTIFY | EventMask::SUBSTRUCTURE_REDIRECT).into();

        //win_aux.event_mask = Some(captured_events); // attempt to let events pass through

        cw(&conn, win_depth, win_id, root,
           0, 0, screen.width_in_pixels, screen.height_in_pixels, 0,
           WindowClass::INPUT_OUTPUT, visual_id, &win_aux).unwrap().check().unwrap();

        let region_id = conn.generate_id()?;
        //conn.xfixes_create_region(region_id, &[]).unwrap().check().unwrap();
        conn.shape_rectangles(SO::SET, SK::INPUT, ClipOrdering::UNSORTED,
                              win_id, 0, 0, &[]).unwrap().check().unwrap();

        x11rb::protocol::xfixes::set_window_shape_region(&conn, win_id, SK::INPUT, 0, 0, region_id);

        let mut gc_aux = CreateGCAux::new();
        gc_aux.background = Some(0);
        gc_aux.foreground = Some(1);
        //gc_aux.plane_mask = Some(1);

        create_gc(&conn, gc_id, win_id, &gc_aux).unwrap().check().unwrap();

        map_window(&conn, win_id).unwrap().check().unwrap();

        let cairo_surface = unsafe {

            let surf = cairo::ffi::cairo_xlib_surface_create(dis, win_id as u64,
                                                             x11_visual.visual, width as c_int, height as c_int);
            cairo::ffi::cairo_xlib_surface_set_size(surf, width as i32, height as i32);

            cairo::Surface::from_raw_full(surf).unwrap()
        };


        let ctx = cairo::Context::new(&cairo_surface).unwrap();

        //let kbg = conn.grab_keyboard(true, win_id, 0u32, GrabMode::ASYNC, GrabMode::ASYNC).unwrap().reply().unwrap();
        //println!("grabbed keyboard: {:?}", &kbg);

        /*
        let loop_starting_time = std::time::Instant::now();

        ctx.set_operator(cairo::Operator::Source);

        let mut j = 0;
        for i in 0.. {
            let pointer = conn.query_pointer(win_id).unwrap().reply().unwrap();
            ctx.push_group();
            ctx.set_source_rgba(0.0, 0.0, 0.0, 0.0);
            ctx.set_operator(cairo::Operator::Source);
            //ctx.fill().unwrap();
            ctx.paint_with_alpha(1.0).unwrap();

            let draw_alpha = ((i as f64)*std::f64::consts::PI/1000.0).sin().abs()*0.5;
            //println!("draw alpha: {}", draw_alpha);

            //ctx.push_group();
            ctx.set_source_rgba(1.0, 0.0, 0.0, draw_alpha);
            //ctx.set_operator(cairo::Operator::Overlay);
            ctx.rectangle(500.0, 500.0, 300.0, 300.0);
            //ctx.clip();
            ctx.fill().unwrap();
            ctx.paint_with_alpha(1.0).unwrap();
            ctx.pop_group_to_source().unwrap();

            ctx.paint_with_alpha(1.0).unwrap();


            cairo_surface.flush();
            conn.flush()?;

            std::thread::sleep(std::time::Duration::from_micros(20));

            j = i;
            if (std::time::Instant::now() - loop_starting_time).as_secs() > 5 {
                break
            }
        }
        */

        Ok(Display{
            conn,
            win_id,
            width,
            height,
            cairo_context: ctx
        })
    }

    pub fn screen_pulse_effect(&mut self, number_of_pulses: u8, color: (f64, f64, f64), alpha: f64) {

        const TICKS_PER_PULSE: u32 = 1000;
        let total_ticks: u32 = TICKS_PER_PULSE*number_of_pulses as u32;

        let loop_starting_time = std::time::Instant::now();

        let ctx = &self.cairo_context;
        let cairo_surface = ctx.target();

        ctx.set_operator(cairo::Operator::Source);

        for i in 0..total_ticks {
            let pointer = self.conn.query_pointer(self.win_id).unwrap().reply().unwrap();
            ctx.push_group();
            ctx.set_source_rgba(0.0, 0.0, 0.0, 0.0);
            ctx.set_operator(cairo::Operator::Source);
            //ctx.fill().unwrap();
            ctx.paint_with_alpha(1.0).unwrap();

            let draw_alpha = ((i as f64)*std::f64::consts::PI/TICKS_PER_PULSE as f64).sin().abs()*alpha;
            //println!("draw alpha: {}", draw_alpha);

            //ctx.push_group();
            ctx.set_source_rgba(color.0, color.1, color.2, draw_alpha);
            //ctx.set_operator(cairo::Operator::Overlay);
            ctx.rectangle(500.0, 500.0, 300.0, 300.0);
            //ctx.clip();
            ctx.fill().unwrap();
            ctx.paint_with_alpha(1.0).unwrap();
            ctx.pop_group_to_source().unwrap();

            ctx.paint_with_alpha(1.0).unwrap();

            cairo_surface.flush();
            self.conn.flush().unwrap();

            std::thread::sleep(std::time::Duration::from_micros(20));

        }
    }

    pub fn default_screen_pulse_effect(&mut self) {
        self.screen_pulse_effect(5, (1.0, 0.0, 0.0), 0.5);
    }

}

impl Drop for Display {
    fn drop(&mut self) {
        self.conn.unmap_window(self.win_id).unwrap().check().unwrap();
    }
}