pub mod applications;

pub trait Provider {
    fn name(&self) -> &'static str;

    /// Filter -> Sort
    fn update_model(&self, query: &str);

    fn view(&self) -> gtk::ListBase;

    fn invoke_action(&self) -> bool;

    fn reset(&self);
}
