use systemd::journal::Journal;

use super::util::filesize;

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Unit(String);

impl super::AggKey for Unit {
  fn new() -> Self {
    Unit(String::new())
  }

  fn examine_record(&mut self, j: &mut Journal) -> systemd::Result<usize> {
    let mut size = 0;
    while let Some(field) = j.enumerate_data()? {
      let r = field.data();
      size += r.len();

      if r.starts_with(b"_SYSTEMD_USER_UNIT=")
        && !r.starts_with(b"_SYSTEMD_USER_UNIT=run-")
        && !r.starts_with(b"_SYSTEMD_USER_UNIT=vte-spawn-")
      {
        self.0.clear();
        let value = r.split(|c| *c == b'=').nth(1).unwrap();
        self.0.push_str(String::from_utf8_lossy(value).as_ref());
        break;
      } else if r.starts_with(b"_SYSTEMD_UNIT=") {
        self.0.clear();
        let value = r.split(|c| *c == b'=').nth(1).unwrap();
        self.0.push_str(String::from_utf8_lossy(value).as_ref());
      }
    }

    if let Some(at) = self.0.find('@') {
      let dot = at + self.0[at..].find('.').unwrap();
      self.0.replace_range(at+1..dot, "");
    }

    Ok(size)
  }

  fn show_result(result: &mut Vec<(Self, usize)>) {
    result.sort_unstable_by_key(|&(_, v)| v);
    for (k, v) in result {
      println!("{:40} {:>9}", k.0, filesize(*v as isize));
    }
  }
}

