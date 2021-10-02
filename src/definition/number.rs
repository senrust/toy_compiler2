pub enum Number{
    U64(u64),
    Double(f64),
}

pub fn string_to_number(string: &String) -> Result<Number, ()> {
    let mut ishex = false;
    let mut isdouble = false;
    let chars: Vec<char> = string.chars().collect();
    if chars.len() > 2 {
        if chars[0] == '0' && (chars[1] == 'x' || chars[1] == 'X') {
            ishex = true;
        }
    }
    if !ishex {
        for ch in &chars {
            if *ch == '.' {
                if isdouble {
            
                    return Err(());
                } else {
                    isdouble = true;
                }
            }
        }
    }

    if isdouble {
        let mut num: f64 = 0.0;
        let mut order: i32 = 0;
        for ch in &chars {
            if *ch == '.' {
                order = -1;
            } else {
                match ch.to_digit(10) {
                    None => return Err(()),
                    Some(digit) => {
                        if order >= 0 {
                            num = num * 10.0 + digit as f64;
                            order += 1;
                        } else {
                            num = num + digit as f64 * 10f64.powi(order as i32);
                            order -= 1;
                        }
                    }
                }
            }
        }
        Ok(Number::Double(num))
    } else {
        let mut radix = 10;
        let mut skipcount = 0;
        if ishex {
            radix = 16;
            skipcount = 2;
        }
        let mut num: u64 = 0;
        for ch in chars.iter().skip(skipcount) {
            match ch.to_digit(radix) {
                None => return Err(()),
                Some(digit) => num = num * radix as u64 + digit as u64,
            }
        }
        Ok(Number::U64(num))
    }
}
