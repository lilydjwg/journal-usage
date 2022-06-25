use std::fmt;

use systemd::journal::Journal;

use super::util::filesize;

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Date(chrono::NaiveDate);

impl fmt::Display for Date {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
    write!(f, "{}", self.0)
  }
}

impl super::AggKey for Date {
  fn new() -> Self {
    Self(chrono::NaiveDate::from_ymd(1970, 1, 1))
  }

  fn examine_record(&mut self, j: &mut Journal) -> systemd::Result<usize> {
    let mut size = 0;
    let t: chrono::DateTime<chrono::Local> = j.timestamp()?.into();
    self.0 = t.date().naive_local();
    while let Some(field) = j.enumerate_data()? {
      let r = field.data();
      size += r.len();
    }

    Ok(size)
  }

  fn show_result(result: &mut Vec<(Self, usize)>) {
    result.sort_unstable_by_key(|(k, _)| k.0);

    use terminal_size::{Width, terminal_size};
    let max: usize = result.iter().map(|(_, v)| *v).max().unwrap_or(1);
    let size = terminal_size();
    let width = if let Some((Width(w), _)) = size { w } else { 0 };
    let width = width.saturating_sub(21) as usize;
    let bar = "*".repeat(width);

    for (k, v) in result {
      if width == 0 {
        println!("{} {:>9}", k, filesize(*v as isize));
      } else {
        let num_stars = width * *v / max;
        println!("{} {:>9} {}", k, filesize(*v as isize), &bar[..num_stars]);
      };
    }
  }
}

