use indicatif::{ProgressBar, ProgressStyle};

use crate::cli::presenter::events::event::Event;

pub struct Presenter {
    progress_bar: Option<ProgressBar>,
}

impl Presenter {
    pub fn new() -> Self {
        Self { progress_bar: None }
    }

    pub fn display<E: Event + ?Sized>(&mut self, event: &E) {
        event.render(self);
    }

    pub fn start_download(&mut self, total_bytes: u64) {
        let pb = ProgressBar::new(total_bytes);

        pb.set_style(
            ProgressStyle::with_template(
                "{spinner:.green} [{elapsed_precise}] [{bar:60.cyan/blue}] {bytes}/{total_bytes} ({eta})"
            )
            .unwrap(),
        );

        self.progress_bar = Some(pb);
    }

    pub fn update_download(&self, bytes: u64) {
        if let Some(pb) = &self.progress_bar {
            pb.set_position(bytes);
        }
    }

    pub fn finish_download(&self) {
        if let Some(pb) = &self.progress_bar {
            pb.finish_with_message("Download complete");
        }
        println!();
    }
}
