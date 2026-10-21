use crate::processing::Orientation;

pub fn encode_orientation(orientation: Orientation) -> String {
    format!(
        r#"{{"pitch":{:.6},"roll":{:.6}}}"#,
        orientation.pitch, orientation.roll
    )
}

#[cfg(test)]
mod tests {
    use crate::processing::Orientation;

    #[test]
    fn encodes_orientation_as_pitch_and_roll_in_radians() {
        assert_eq!(
            super::encode_orientation(Orientation {
                pitch: 0.25,
                roll: -0.5,
            }),
            r#"{"pitch":0.250000,"roll":-0.500000}"#
        );
    }
}
