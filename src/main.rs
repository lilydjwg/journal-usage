use std::collections::HashMap;
use systemd::journal;

fn next_message(j: &mut journal::Journal) -> systemd::Result<Option<journal::JournalRecord>> {
  loop {
    match j.next_record() {
      Err(e) if e.raw_os_error() == Some(74) => continue,
      r => return r,
    }
  }
}

fn main() -> systemd::Result<()> {
  let mut j = journal::Journal::open(
    journal::JournalFiles::All,
    false, true)?;
  let mut m = HashMap::new();

  while let Some(mut r) = next_message(&mut j)? {
    let mut service = match r.remove("_SYSTEMD_USER_UNIT") {
      Some(u) if !u.starts_with("run-") => u,
      _ => r.remove("_SYSTEMD_UNIT").unwrap_or_else(
        ||r.remove("_TRANSPORT").unwrap()),
    };

    if let Some(at) = service.find('@') {
      let dot = service.find('.').unwrap();
      service.replace_range(at..dot, "");
    }

    let amt = r.get("MESSAGE").unwrap().len();
    let c = m.entry(service).or_insert(0);
    *c += amt;
  }

  let mut data: Vec<(String, usize)> = m.into_iter().collect();
  data.sort_unstable_by_key(|&(_, v)| v);
  for (k, v) in data {
    println!("{:30} {}", k, filesize(v as isize));
  }

  Ok(())
}

const UNITS: [char; 4] = ['K', 'M', 'G', 'T'];

fn filesize(size: isize) -> String {
  let mut left = size.abs() as f64;
  let mut unit = -1;

  while left > 1100. && unit < 3 {
    left /= 1024.;
    unit += 1;
  }
  if unit == -1 {
    format!("{}B", size)
  } else {
    if size < 0 {
      left = -left;
    }
    format!("{:.1}{}iB", left, UNITS[unit as usize])
  }
}

