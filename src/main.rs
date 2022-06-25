use std::collections::HashMap;
use systemd::journal;

mod per_unit;
mod per_date;
mod util;

fn main() -> systemd::Result<()> {
  let arg1 = std::env::args().nth(1);
  let arg1: Option<&str> = arg1.as_deref();
  match arg1 {
    Some("unit") | None => process::<per_unit::Unit>(),
    Some("date") => process::<per_date::Date>(),
    _ => {
      eprintln!("unrecognized argument.");
      std::process::exit(1);
    },
  }
}

trait AggKey: Clone + Eq + std::hash::Hash {
  fn new() -> Self;
  fn examine_record(
    &mut self, j: &mut journal::Journal,
  ) -> systemd::Result<usize>;
  fn show_result(result: &mut Vec<(Self, usize)>);
}

fn process<K: AggKey>() -> systemd::Result<()> {
  let mut j = journal::OpenOptions::default()
    .local_only(true)
    .open()?;

  let mut m = HashMap::new();
  let mut key = K::new();

  while j.next()? != 0 {
    let size = key.examine_record(&mut j)?;

    if let Some(c) = m.get_mut(&key) {
      *c += size;
    } else {
      m.insert(key.clone(), size);
    }
  }

  let mut data: Vec<(K, usize)> = m.into_iter().collect();
  K::show_result(&mut data);

  Ok(())
}

