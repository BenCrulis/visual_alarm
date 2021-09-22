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


pub fn slice_to_sequence_buffer(slice: &[u8]) -> [u8; 32] {
    let mut buffer = [0; 32];
    for (i, v) in slice.iter().enumerate() {
        buffer[i] = *v;
    }
    buffer
}
