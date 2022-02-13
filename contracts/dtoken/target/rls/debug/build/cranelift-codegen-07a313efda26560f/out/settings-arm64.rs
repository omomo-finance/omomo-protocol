#[derive(Clone)]
/// Flags group `arm64`.
pub struct Flags {
    bytes: [u8; 0],
}
impl Flags {
    /// Create flags arm64 settings group.
    #[allow(unused_variables)]
    pub fn new(shared: &settings::Flags, builder: Builder) -> Self {
        let bvec = builder.state_for("arm64");
        let mut arm64 = Self { bytes: [0; 0] };
        debug_assert_eq!(bvec.len(), 0);
        arm64.bytes[0..0].copy_from_slice(&bvec);
        arm64
    }
}
/// User-defined settings.
#[allow(dead_code)]
impl Flags {
    /// Get a view of the boolean predicates.
    pub fn predicate_view(&self) -> crate::settings::PredicateView {
        crate::settings::PredicateView::new(&self.bytes[0..])
    }
}
static DESCRIPTORS: [detail::Descriptor; 0] = [
];
static ENUMERATORS: [&str; 0] = [
];
static HASH_TABLE: [u16; 1] = [
    0xffff,
];
static PRESETS: [(u8, u8); 0] = [
];
static TEMPLATE: detail::Template = detail::Template {
    name: "arm64",
    descriptors: &DESCRIPTORS,
    enumerators: &ENUMERATORS,
    hash_table: &HASH_TABLE,
    defaults: &[],
    presets: &PRESETS,
};
/// Create a `settings::Builder` for the arm64 settings group.
pub fn builder() -> Builder {
    Builder::new(&TEMPLATE)
}
impl fmt::Display for Flags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "[arm64]")?;
        for d in &DESCRIPTORS {
            if !d.detail.is_preset() {
                write!(f, "{} = ", d.name)?;
                TEMPLATE.format_toml_value(d.detail, self.bytes[d.offset as usize], f)?;
                writeln!(f)?;
            }
        }
        Ok(())
    }
}
