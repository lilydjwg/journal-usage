use std::collections::HashMap;
use systemd::journal;

fn handle_one_record(
  j: &mut journal::Journal,
  service: &mut String,
) -> systemd::Result<usize> {
  let mut size = 0;

  while let Some(field) = j.enumerate_data()? {
    let r = field.data();
    size += r.len();

    if r.starts_with(b"_SYSTEMD_USER_UNIT=") && !r.starts_with(b"_SYSTEMD_USER_UNIT=run-") {
      service.clear();
      let value = r.split(|c| *c == b'=').nth(1).unwrap();
      service.push_str(String::from_utf8_lossy(value).as_ref());
    } else if r.starts_with(b"_SYSTEMD_UNIT=") && service.is_empty() {
      let value = r.split(|c| *c == b'=').nth(1).unwrap();
      service.push_str(String::from_utf8_lossy(value).as_ref());
    }
  }

  if let Some(at) = service.find('@') {
    let dot = service.find('.').unwrap();
    service.replace_range((at+1)..dot, "");
  }

  Ok(size)
}

fn main() -> systemd::Result<()> {
  let mut j = journal::OpenOptions::default()
    .local_only(true)
    .open()?;

  let mut m = HashMap::new();
  let mut service = String::new();

  while j.next()? != 0 {
    let size = handle_one_record(&mut j, &mut service)?;

    if let Some(c) = m.get_mut(&service) {
      *c += size;
    } else {
      m.insert(service.clone(), size);
    }
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

