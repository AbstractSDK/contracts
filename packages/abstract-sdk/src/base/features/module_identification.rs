/// Return the identifier for this module.
pub trait ModuleIdentification: Sized {
    fn module_id(&self) -> &'static str;
}
