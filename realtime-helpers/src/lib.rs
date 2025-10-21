pub fn xdp_obj_path() -> &'static str {
    include_str!(concat!(env!("OUT_DIR"), "/xdp_obj_path.txt"))
}
