include!(concat!(env!("OUT_DIR"), "/xdp_obj_path.rs"));

pub fn xdp_obj_path() -> &'static str {
    XDP_OBJ_PATH
}

pub mod xdp;
