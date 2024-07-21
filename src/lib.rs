pub mod ser;
pub mod data;


/* from spreadsheet */
const PERCENTS: [(f32, f32); 15] = [
    (3.0, 0.0),
    (3.1, 0.8),
    (3.2, 1.2),
    (3.3, 2.0),
    (3.4, 4.0),
    (3.5, 12.0),
    (3.6, 20.0),
    (3.7, 33.0),
    (3.8, 59.0),
    (3.9, 73.0),
    (4.0, 85.0),
    (4.1, 96.0),
    (4.2, 100.0),
    (4.25, 105.0),
    (4.3, 110.0),
];

fn round_v(v: f32) -> f32 {
    (v * 10.0).round() as f32 / 10.0
}

pub fn v_to_percent(value: f32) -> Option<f32> {
    let mut iter = PERCENTS.iter()
        .zip(PERCENTS.iter().skip(1))
        .skip_while(|((_v1, _), (v2, _))| *v2 < value);

    if let Some(((start_v, start_pct), (end_v, end_pct))) = iter.next() {
        let pct = (value - start_v) / (end_v - start_v);
        return Some(round_v(pct * (end_pct - start_pct) + start_pct));
    }

    // value outside range
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v_to_percent_test() {
        assert_eq!(47.3, v_to_percent(3.745).unwrap());
    }
}

