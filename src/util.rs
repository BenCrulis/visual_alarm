use x11rb::protocol::xproto::{Depth, VisualClass, Visualtype};
use std::os::raw::c_int;


pub fn find_visual_with_depth(visuals: &[Depth], depth: u8, visual_class: VisualClass) -> Option<Visualtype> {
    for depth_object in visuals {
        if depth_object.depth == depth {
            for visualtype in &depth_object.visuals {
                if visualtype.class == visual_class {
                    return Some(visualtype.clone())
                }
            }
        }
    }
    None
}


pub fn visual_type_to_x11visual(visualtype: Visualtype, ext_data: *mut x11::xlib::XExtData) -> x11::xlib::Visual {
    let visual_class: u8 = visualtype.class.into();
    x11::xlib::Visual{
        ext_data,
        visualid: visualtype.visual_id as u64,
        class: visual_class as c_int,
        red_mask: visualtype.red_mask as u64,
        green_mask: visualtype.green_mask as u64,
        blue_mask: visualtype.blue_mask as u64,
        bits_per_rgb: visualtype.bits_per_rgb_value as c_int,
        map_entries: visualtype.colormap_entries as c_int
    }
}


pub fn slice_to_sequence_buffer(slice: &[u8]) -> [u8; 32] {
    let mut buffer = [0; 32];
    for (i, v) in slice.iter().enumerate() {
        buffer[i] = *v;
    }
    buffer
}
