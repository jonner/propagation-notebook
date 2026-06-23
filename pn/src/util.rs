use indicatif::ProgressBar;
use libpropagation::ImportProgressReporter;

pub(crate) fn join_or_default<T, F>(items: &[T], default: &str, extract: F) -> String
where
    F: Fn(&T) -> String,
{
    if items.is_empty() {
        default.to_string()
    } else {
        items.iter().map(extract).collect::<Vec<_>>().join("\n")
    }
}

#[derive(Debug, Default)]
pub struct IndicatifImportProgress {
    pb: Option<ProgressBar>,
}

impl ImportProgressReporter for IndicatifImportProgress {
    fn begin_step(&mut self, name: &str, total: usize) {
        println!("{name}");
        self.pb = Some(ProgressBar::new(total as u64));
    }

    fn increment(&mut self) {
        if let Some(pb) = &self.pb {
            pb.inc(1);
        }
    }

    fn finish_step(&mut self) {
        if let Some(pb) = self.pb.take() {
            pb.finish_and_clear();
        }
    }
}
