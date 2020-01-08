use std::collections::HashMap;
use systemd::journal;

fn main() -> systemd::Result<()> {
  let mut j = journal::Journal::open(
    journal::JournalFiles::All,
    false, true)?;
  let mut m = HashMap::new();

  loop {
    let mut service = String::new();
    let mut size = 0;
    match j.next_record_raw(|r| {
      size += r.len();

      if r.starts_with(b"_SYSTEMD_USER_UNIT=") &&
        !r.starts_with(b"_SYSTEMD_USER_UNIT=run-") {
          service.clear();
          let value = r.split(|c| *c == b'=').nth(1).unwrap();
          service.push_str(String::from_utf8_lossy(value).as_ref());
      } else if r.starts_with(b"_SYSTEMD_UNIT=") && service.is_empty() {
          let value = r.split(|c| *c == b'=').nth(1).unwrap();
          service.push_str(String::from_utf8_lossy(value).as_ref());
      }
    }) {
      Ok(None) => break,
      Err(e) if e.raw_os_error() == Some(74) => continue,
      e => e?,
    };

    if let Some(at) = service.find('@') {
      let dot = service.find('.').unwrap();
      service.replace_range((at+1)..dot, "");
    }

    let c = m.entry(service).or_insert(0);
    *c += size;
  }

  let mut data: Vec<(String, usize)> = m.into_iter().collect();
  data.sort_unstable_by_key(|&(_, v)| v);
  for (k, v) in data {
    println!("{:40} {:>9}", k, filesize(v as isize));
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

