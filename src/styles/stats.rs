#[derive(Debug, Default, Clone, Copy)]
pub struct CascadeStats {
    pub matches_static: usize,
    pub matches_dynamic: usize,
    pub apply_ok: usize,
    pub apply_error: usize,
}
