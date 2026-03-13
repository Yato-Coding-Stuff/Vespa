use crate::cli::presenter::presenter::Presenter;

pub trait Event {
    fn render(&self, presenter: &mut Presenter);
}
