pub trait StructDeepGetter<T> {
    fn deeper_structs() -> Vec<String>;
    fn get_path(&self, path: &str) -> T;
}
