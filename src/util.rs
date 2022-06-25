const UNITS: [char; 4] = ['K', 'M', 'G', 'T'];

pub fn filesize(size: isize) -> String {
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

