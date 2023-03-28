use log::error;

use crate::Source;

/// Parse a formatted string.
///
/// Arguments:
/// - `fmt: &str`: the format string (with '%' characters).
/// - `args: &[u8]`: bytes containing argument values after the argument pointer.
/// - `mem: impl Source<u8>`: a reference to the cart's memory. This is used to access inserted strings.
pub fn tracef<T: Source<u8>>(fmt: &str, args: &[u8], mem: &T) -> String {
    let mut arg_idx = 0;
    let mut fmt = fmt.chars();

    let mut output = String::new();

    while let Some(ch) = fmt.next() {
        // break on null terminated string.
        if ch == '\0' {
            break;
        }

        // characters other than formatting character '%' are just added.
        if ch != '%' {
            output.push(ch);
            continue;
        }

        // if we've hit the formatting character ('%') then move to the next
        // char and check to see what it is and replace it with the value it references.
        if let Some(ch) = fmt.next() {
            match ch {
                'c' => {
                    let val: u32 = match args
                        .get(arg_idx..(arg_idx + 4))
                        .map(|s| s.try_into().unwrap())
                    {
                        Some(bytes) => u32::from_le_bytes(bytes),
                        None => {
                            error!("failed to read char at {arg_idx}");
                            break;
                        }
                    };

                    output.push(char::from_u32(val).unwrap_or('!'));
                    arg_idx += 4;
                }
                'd' | 'x' => {
                    let val: i32 = match args
                        .get(arg_idx..(arg_idx + 4))
                        .map(|s| s.try_into().unwrap())
                    {
                        Some(bytes) => i32::from_le_bytes(bytes),
                        None => {
                            error!("failed to read i32 at {arg_idx}");
                            break;
                        }
                    };

                    output.push_str(&val.to_string());
                    arg_idx += 4;
                }
                's' => {
                    let mut str_ptr: u32 = match args
                        .get(arg_idx..(arg_idx + 4))
                        .map(|s| s.try_into().unwrap())
                    {
                        Some(bytes) => u32::from_le_bytes(bytes),
                        None => {
                            error!("failed to read ptr at {arg_idx}");
                            break;
                        }
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
                    arg_idx += 4;
                }
                'f' => {
                    let val: f64 = match args
                        .get(arg_idx..(arg_idx + 8))
                        .map(|s| s.try_into().unwrap())
                    {
                        Some(bytes) => f64::from_le_bytes(bytes),
                        None => {
                            error!("failed to read f64 at {arg_idx}");
                            break;
                        }
                    };

                    output.push_str(&val.to_string());
                    arg_idx += 8;
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
            tracef("%f;%f", bytemuck::cast_slice(&[0.473, 0.856]), &[])
        );
    }

    #[test]
    fn tracef_int() {
        assert_eq!(
            "4082;8088",
            tracef("%d;%d", bytemuck::cast_slice(&[4082i32, 8088i32]), &[])
        )
    }

    #[test]
    fn tracef_str() {
        assert_eq!(
            "here's your str: 'inner string!'",
            tracef(
                "here's your str: '%s'",
                bytemuck::cast_slice(&[11u32]),
                &"before the inner string!".as_bytes().to_vec()
            )
        )
    }

    #[test]
    fn tracef_char() {
        assert_eq!(
            "exclamation mark: !; ampersand: &",
            tracef(
                "exclamation mark: %c; ampersand: %c",
                bytemuck::cast_slice::<char, u8>(&['!', '&']),
                &[]
            )
        )
    }
}
