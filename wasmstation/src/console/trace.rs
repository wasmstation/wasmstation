use crate::Source;

/// Parse a formatted string.
///
/// Arguments:
///  - `fmt: usize`: a pointer to the format string (**null terminated ASCII**).
///  - `args: usize`: a pointer to the data arguments.
///  - `mem: &impl Source<u8>`: a reference to the game's memory.
pub fn tracef<T: Source<u8>>(mut fmt: usize, mut args: usize, mem: &T) -> String {
    let mut fmt_str = String::new();

    while let Some(b) = mem.item_at(fmt) {
        if b == 0 {
            break;
        }

        fmt_str.push(char::from_u32(b.into()).unwrap_or('!'));

        fmt += 1;
    }

    let mut fmt_chars = fmt_str.chars();
    let mut output = String::new();

    while let Some(ch) = fmt_chars.next() {
        // break for null terminated string.
        if ch == '\0' {
            break;
        }

        // characters other than the formatting character '%' are just added.
        if ch != '%' {
            output.push(ch);
            continue;
        }

        // if we've hit the formatting character ('%') then move to the next
        // char and check to see what it is and replace it with the value it references.
        if let Some(ch) = fmt_chars.next() {
            match ch {
                'c' => {
                    let val = match mem.items_at(args) {
                        Some(val) => u32::from_le_bytes(val),
                        None => continue,
                    };

                    output.push(val.try_into().unwrap_or('!'));
                    args += 4;
                }
                'd' | 'x' => {
                    let val = match mem.items_at(args) {
                        Some(bytes) => i32::from_le_bytes(bytes),
                        None => continue,
                    };

                    output.push_str(&val.to_string());
                    args += 4;
                }
                's' => {
                    let mut str_ptr = match mem.items_at(args) {
                        Some(val) => u32::from_le_bytes(val),
                        None => continue,
                    };

                    let mut nstr = String::new();

                    while let Some(byte) = mem.item_at(str_ptr as usize) {
                        if byte == 0 {
                            break;
                        }

                        nstr.push(byte.try_into().unwrap_or('!'));
                        str_ptr += 1;
                    }

                    output.push_str(&nstr);
                    args += 4;
                }
                'f' => {
                    let val = match mem.items_at(args) {
                        Some(val) => f64::from_le_bytes(val),
                        None => continue,
                    };

                    output.push_str(&val.to_string());
                    args += 8;
                }
                _ => output.push(ch),
            }
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::tracef;

    #[test]
    fn tracef_float() {
        assert_eq!(
            "0.473;0.856",
            tracef(
                16,
                0,
                &[
                    &bytemuck::cast_slice(&[0.473_f64, 0.856_f64]),
                    "%f;%f\0".as_bytes(),
                ]
                .concat()
            )
        );
    }

    #[test]
    fn tracef_int() {
        assert_eq!(
            "4082;8088",
            tracef(
                0,
                6,
                &[
                    "%d;%d\0".as_bytes(),
                    &bytemuck::cast_slice(&[4082_i32, 8088_i32])
                ]
                .concat()
            )
        )
    }

    #[test]
    fn tracef_str() {
        assert_eq!(
            "here's your str: 'the inner string!'",
            tracef(
                21,
                43,
                &[
                    "the inner string!\0---here's your str: '%s'\0".as_bytes(),
                    &bytemuck::cast_slice(&[0_u32])
                ]
                .concat()
            )
        )
    }

    #[test]
    fn tracef_char() {
        assert_eq!(
            "exclamation mark: !; ampersand: &",
            tracef(
                0,
                36,
                &[
                    "exclamation mark: %c; ampersand: %c\0".as_bytes(),
                    &bytemuck::cast_slice(&['!', '&'])
                ]
                .concat(),
            )
        )
    }
}
