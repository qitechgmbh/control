use ethercrab::{error::Error, SubDevice, SubDevicePdi, SubDeviceRef};

pub async fn u16dump<'maindevice, 'group>(
    subdevice: &SubDeviceRef<'maindevice, &SubDevice>,
    start_word: u16,
    end_word: u16,
) -> Result<(), Error> {
    let eeprom = subdevice.eeprom();
    let mut words: Vec<u16> = Vec::new();
    for word in start_word..end_word {
        words.push(eeprom.read(word).await?);
    }

    // print like this: address words and then ascii
    // 0000 | 0000 0001 0002 0003 0004 0005 0006 0007 | .. .. .. .. .. .. .. ..
    // 0010 | 0008 0009 000a 000b 000c 000d 000e 000f | .. .. .. .. .. .. .. ..
    // 0020 | 0010 0011 0012 0013 0014 0015 0016 0017 | .. .. .. .. .. .. .. ..
    // 0030 | 0018 0019 001a 001b 001c 001d 001e 001f | .. .. .. .. .. .. .. ..
    // 0040 | 0020 0021 0022 0023 0024 0025 0026 0027 | .. .. .. .. .. .. .. ..

    u16print(start_word, end_word, words);

    Ok(())
}

fn u16print(start_word: u16, end_word: u16, data: Vec<u16>) {
    let table_start_word = start_word & 0xfff0;
    let table_end_word = (end_word & 0xfff0_u16) + 0x10_u16;

    let rows = table_end_word - table_start_word >> 4;

    for row in 0..rows {
        print!("0x{:04x} | ", (table_start_word + row * 0x10) / 2);
        for word in 0..8 {
            let word_address = row * 8 + word;
            if word_address < start_word {
                print!("     ");
            } else {
                let i = (word_address - start_word) as usize;
                if i > data.len() - 1 {
                    print!("     ");
                } else {
                    print!("{:04x} ", data[i]);
                }
            }
        }
        print!("\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hexprint() {
        let data = vec![0x0000, 0x1ced];
        u16print(0x01, 0x40, data);
    }
}
