use x11rb::connect;
use x11rb::rust_connection::RustConnection;
use x11rb::connection::{RequestConnection, Connection};
use x11rb::protocol::xproto::{Screen, VisualClass, ColormapAlloc, create_colormap,
                              CreateWindowAux, WindowClass, ClipOrdering, CreateGCAux, create_gc, map_window, ConnectionExt, Gcontext};
use x11rb::protocol::xproto::create_window as  cw;
use x11rb::protocol::shape::{SO, SK};
use x11rb::protocol::shape::ConnectionExt as _;

use crate::util::find_visual_with_depth;

pub struct Display {
    conn: RustConnection,
    win_id: u32,
    width: u16,
    height: u16,
    gc_context: Gcontext
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

        //let vinfo = get_window_attributes(&conn, root).unwrap().reply().unwrap();

        let win_id = conn.generate_id().unwrap();
        let gc_id = conn.generate_id().unwrap();
        let cm_id = conn.generate_id().unwrap();

        let win_depth = 32;
        let visual_ob = find_visual_with_depth(&screen.allowed_depths, win_depth, VisualClass::TRUE_COLOR).unwrap();
        let visual_id = visual_ob.visual_id;

        create_colormap(&conn,ColormapAlloc::NONE,cm_id,root, visual_id).unwrap().check().unwrap();

        let mut win_aux = CreateWindowAux::new();
        //win_aux.background_pixel = Some(0);
        win_aux.override_redirect = Some(1);
        win_aux.border_pixel = Some(1);
        win_aux.event_mask = Some(0);
        win_aux.colormap = Some(cm_id);
        //win_aux.background_pixmap = None;

        /*
        let captured_events: u32 = (EventMask::POINTER_MOTION | EventMask::POINTER_MOTION_HINT | EventMask::BUTTON_PRESS
            | EventMask::KEY_PRESS
            | EventMask::KEY_RELEASE | EventMask::STRUCTURE_NOTIFY | EventMask::SUBSTRUCTURE_REDIRECT).into();

         */

        //win_aux.event_mask = Some(captured_events); // attempt to let events pass through

        cw(&conn, win_depth, win_id, root,
           0, 0, screen.width_in_pixels, screen.height_in_pixels, 0,
           WindowClass::INPUT_OUTPUT, visual_id, &win_aux).unwrap().check().unwrap();

        let region_id = conn.generate_id()?;
        //conn.xfixes_create_region(region_id, &[]).unwrap().check().unwrap();
        conn.shape_rectangles(SO::SET, SK::INPUT, ClipOrdering::UNSORTED,
                              win_id, 0, 0, &[]).unwrap().check().unwrap();

        x11rb::protocol::xfixes::set_window_shape_region(&conn, win_id, SK::INPUT, 0, 0, region_id).unwrap();

        let mut gc_aux = CreateGCAux::new();
        //gc_aux.background = Some(0);
        //gc_aux.foreground = Some(0);
        //gc_aux.function = Some(x11rb::protocol::xproto::GX::COPY);
        gc_aux.plane_mask = Some(u32::MAX);


        create_gc(&conn, gc_id, win_id, &gc_aux).unwrap().check().unwrap();

        map_window(&conn, win_id).unwrap().check().unwrap();

        Ok(Display{
            conn,
            win_id,
            width,
            height,
            gc_context: gc_id
        })
    }

    pub fn screen_pulse_effect(&mut self, number_of_pulses: u8, color: (f64, f64, f64), alpha: f64) {

        const TICKS_PER_PULSE: u32 = 200;
        let total_ticks: u32 = TICKS_PER_PULSE*number_of_pulses as u32;

        //let loop_starting_time = std::time::Instant::now();

        for i in 0..total_ticks {
            //let pointer = self.conn.query_pointer(self.win_id).unwrap().reply().unwrap();

            let draw_alpha = ((i as f64)*std::f64::consts::PI/TICKS_PER_PULSE as f64).sin().abs()*alpha*255.0;
            //println!("draw alpha: {}", draw_alpha);

            /*
            self.conn.put_image(ImageFormat::Z_PIXMAP, self.win_id, self.gc_context,
                                self.width, self.height, 0, 0, 0, self.depth, &rect);

             */

            let mut ch_gc = x11rb::protocol::xproto::ChangeGCAux::new();

            let red = (color.0 * draw_alpha) as u32;
            let green = (color.1 * draw_alpha) as u32;
            let blue = (color.2 * draw_alpha) as u32;
            let color: u32 = (blue << 0) | (green << 8) | (red << 16) | ((draw_alpha as u32) << 24);

            ch_gc.foreground = Some(color);

            //println!("color: {:032b}", color);

            self.conn.change_gc(self.gc_context, &ch_gc).unwrap().check().unwrap();

            //self.conn.clear_area(false, self.win_id, 0, 0, self.width, self.height).unwrap().check().unwrap();
            self.conn.poly_fill_rectangle(self.win_id, self.gc_context, &[x11rb::protocol::xproto::Rectangle{
                x: 0,
                y: 0,
                width: self.width,
                height: self.height
            }]).unwrap();

            self.conn.flush().unwrap();

            std::thread::sleep(std::time::Duration::from_micros(5000));
            //std::thread::sleep(std::time::Duration::from_secs(1));

        }
    }

}

impl Drop for Display {
    fn drop(&mut self) {
        self.conn.unmap_window(self.win_id).unwrap().check().unwrap();
    }
}