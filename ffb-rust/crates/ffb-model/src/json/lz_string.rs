//! 1:1 translation of `com.fumbbl.ffb.json.LZString` (lzstring4j, from pieroxy's lz-string).
//!
//! The FFB client and server compress every WebSocket command with `compressToUTF16` and send the
//! result UTF-8 encoded in a binary frame (`CommandEndpoint.java:118-130`,
//! `ServerCommunication.java:330-345`), so this has to be bit-exact with the Java class.
//!
//! Java `String`s are UTF-16; every function here works on UTF-16 code units (`u16`) exactly like the
//! Java `charAt`/`(char)` arithmetic, and the `&str` wrappers convert at the edge.
//! Where Java would throw (`StringIndexOutOfBoundsException` on a malformed stream, a `null` return),
//! the Rust function returns `None`.
use std::collections::{HashMap, HashSet};

/// Java: `static String keyStr`.
const KEY_STR: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/=";

/// The bit writer Java inlines at every output site:
/// `context_data_val = (context_data_val << 1) | bit; if (position == 15) {...} else position++`.
struct BitWriter {
    data_string: Vec<u16>,
    data_val: i32,
    data_position: i32,
}

impl BitWriter {
    fn push(&mut self, bit: i32) {
        self.data_val = (self.data_val << 1) | bit;
        if self.data_position == 15 {
            self.data_position = 0;
            self.data_string.push(self.data_val as u16);
            self.data_val = 0;
        } else {
            self.data_position += 1;
        }
    }
}

/// Java: the `context_enlargeIn--; if (Double.valueOf(context_enlargeIn).intValue() == 0) {...}` block.
fn enlarge(enlarge_in: &mut f64, num_bits: &mut i32) {
    *enlarge_in -= 1.0;
    if (*enlarge_in as i32) == 0 {
        *enlarge_in = 2f64.powi(*num_bits);
        *num_bits += 1;
    }
}

/// Java: the "output the code for w" block shared by the loop body and the tail of `compress`.
fn output_w(
    w: &[u16],
    dictionary: &HashMap<Vec<u16>, i32>,
    dictionary_to_create: &mut HashSet<Vec<u16>>,
    out: &mut BitWriter,
    enlarge_in: &mut f64,
    num_bits: &mut i32,
) {
    if dictionary_to_create.contains(w) {
        if (w[0] as i32) < 256 {
            for _ in 0..*num_bits {
                out.push(0);
            }
            let mut value = w[0] as i32;
            for _ in 0..8 {
                out.push(value & 1);
                value >>= 1;
            }
        } else {
            let mut value = 1;
            for _ in 0..*num_bits {
                out.push(value);
                value = 0;
            }
            let mut value = w[0] as i32;
            for _ in 0..16 {
                out.push(value & 1);
                value >>= 1;
            }
        }
        enlarge(enlarge_in, num_bits);
        dictionary_to_create.remove(w);
    } else {
        let mut value = dictionary[w];
        for _ in 0..*num_bits {
            out.push(value & 1);
            value >>= 1;
        }
    }
    enlarge(enlarge_in, num_bits);
}

/// Java: `public static String compress(String uncompressed)`.
pub fn compress(uncompressed: &[u16]) -> Vec<u16> {
    let mut context_dictionary: HashMap<Vec<u16>, i32> = HashMap::new();
    let mut context_dictionary_to_create: HashSet<Vec<u16>> = HashSet::new();
    let mut context_w: Vec<u16> = Vec::new();
    let mut context_enlarge_in = 2f64; // Compensate for the first entry which should not count
    let mut context_dict_size = 3;
    let mut context_num_bits = 2;
    let mut out = BitWriter { data_string: Vec::new(), data_val: 0, data_position: 0 };

    for &ch in uncompressed {
        let context_c = vec![ch];
        if !context_dictionary.contains_key(&context_c) {
            context_dictionary.insert(context_c.clone(), context_dict_size);
            context_dict_size += 1;
            context_dictionary_to_create.insert(context_c.clone());
        }

        let mut context_wc = context_w.clone();
        context_wc.push(ch);

        if context_dictionary.contains_key(&context_wc) {
            context_w = context_wc;
        } else {
            output_w(&context_w, &context_dictionary, &mut context_dictionary_to_create, &mut out,
                     &mut context_enlarge_in, &mut context_num_bits);
            // Add wc to the dictionary.
            context_dictionary.insert(context_wc, context_dict_size);
            context_dict_size += 1;
            context_w = context_c;
        }
    }

    // Output the code for w.
    if !context_w.is_empty() {
        output_w(&context_w, &context_dictionary, &mut context_dictionary_to_create, &mut out,
                 &mut context_enlarge_in, &mut context_num_bits);
    }

    // Mark the end of the stream
    let mut value = 2;
    for _ in 0..context_num_bits {
        out.push(value & 1);
        value >>= 1;
    }

    // Flush the last char
    loop {
        out.data_val <<= 1;
        if out.data_position == 15 {
            out.data_string.push(out.data_val as u16);
            break;
        } else {
            out.data_position += 1;
        }
    }
    out.data_string
}

/// Java: `public static String decompressHexString(String hexString)`.
pub fn decompress_hex_string(hex_string: &str) -> Option<Vec<u16>> {
    let bytes = hex_string.as_bytes();
    if bytes.len() % 2 != 0 {
        // Java: throw new RuntimeException("Input string length should be divisible by two")
        return None;
    }
    let mut int_arr = Vec::with_capacity(bytes.len() / 2);
    for pair in bytes.chunks(2) {
        int_arr.push(u32::from_str_radix(std::str::from_utf8(pair).ok()?, 16).ok()?);
    }
    let mut sb: Vec<u16> = Vec::new();
    let mut j = 0;
    while j < int_arr.len() {
        // Java: sb.append(Character.toChars(intArr[j] | intArr[j + 1] << 8))
        let cp = int_arr[j] | (*int_arr.get(j + 1)? << 8);
        sb.push(cp as u16);
        j += 2;
    }
    decompress(&sb)
}

/// Java: class `Data` (the reader state of `decompress`).
struct Data<'a> {
    val: i32,
    string: &'a [u16],
    position: i32,
    index: usize,
}

impl Data<'_> {
    /// Java's inlined `while (power != maxpower) {...}` bit reader; `None` where Java's
    /// `data.string.charAt(data.index++)` would throw.
    fn read_bits(&mut self, num_bits: i32) -> Option<i32> {
        let mut bits = 0;
        let maxpower = 2f64.powi(num_bits) as i32;
        let mut power = 1;
        while power != maxpower {
            let resb = self.val & self.position;
            self.position >>= 1;
            if self.position == 0 {
                self.position = 32768;
                self.val = *self.string.get(self.index)? as i32;
                self.index += 1;
            }
            bits |= (if resb > 0 { 1 } else { 0 }) * power;
            power <<= 1;
        }
        Some(bits)
    }
}

/// Java: `public static String decompress(String compressed)`. `None` = Java `null` or a throw.
pub fn decompress(compressed: &[u16]) -> Option<Vec<u16>> {
    if compressed.is_empty() {
        return None;
    }
    let mut dictionary: Vec<Vec<u16>> = Vec::with_capacity(200);
    let mut enlarge_in = 4f64;
    let mut dict_size = 4usize;
    let mut num_bits = 3;
    let mut data = Data { val: compressed[0] as i32, string: compressed, position: 32768, index: 1 };

    for _ in 0..3 {
        dictionary.push(Vec::new());
    }

    let c: Vec<u16> = match data.read_bits(2)? {
        0 => vec![data.read_bits(8)? as u16],
        1 => vec![data.read_bits(16)? as u16],
        2 => return Some(Vec::new()),
        _ => Vec::new(),
    };

    dictionary.push(c.clone());
    let mut w = c.clone();
    let mut result: Vec<u16> = c;

    loop {
        if data.index > data.string.len() {
            return Some(Vec::new());
        }

        let bits = data.read_bits(num_bits)?;
        let mut d = bits as usize;
        match bits {
            0 | 1 => {
                let width = if bits == 0 { 8 } else { 16 };
                let ch = data.read_bits(width)? as u16;
                dictionary.push(vec![ch]);
                dict_size += 1;
                d = dict_size - 1;
                enlarge_in -= 1.0;
            }
            2 => return Some(result),
            _ => {}
        }

        if (enlarge_in as i32) == 0 {
            enlarge_in = 2f64.powi(num_bits);
            num_bits += 1;
        }

        let entry: Vec<u16> = if d < dictionary.len() {
            dictionary[d].clone()
        } else if d == dict_size {
            let mut e = w.clone();
            e.push(*w.first()?);
            e
        } else {
            return None;
        };

        result.extend_from_slice(&entry);

        // Add w+entry[0] to the dictionary.
        let mut add = w.clone();
        add.push(*entry.first()?);
        dictionary.push(add);
        dict_size += 1;
        enlarge_in -= 1.0;

        w = entry;

        if (enlarge_in as i32) == 0 {
            enlarge_in = 2f64.powi(num_bits);
            num_bits += 1;
        }
    }
}

/// Java: `public static String compressToUTF16(String input)`.
pub fn compress_to_utf16_units(input: &[u16]) -> Vec<u16> {
    let mut output: Vec<u16> = Vec::new();
    let mut current: i32 = 0;
    let mut status = 0;

    let input = compress(input);

    for &ch in &input {
        let c = ch as i32;
        let s = status;
        status += 1;
        match s {
            0 => {
                output.push(((c >> 1) + 32) as u16);
                current = (c & 1) << 14;
            }
            14 => {
                output.push(((current + (c >> 15)) + 32) as u16);
                output.push(((c & 32767) + 32) as u16);
                status = 0;
            }
            k => {
                // Java cases 1..13: output (current + (c >> (k+1))) + 32; current = (c & (2^(k+1)-1)) << (14-k)
                let shift = k + 1;
                output.push(((current + (c >> shift)) + 32) as u16);
                current = (c & ((1 << shift) - 1)) << (14 - k);
            }
        }
    }

    output.push((current + 32) as u16);
    output
}

/// Java: `public static String decompressFromUTF16(String input)`.
pub fn decompress_from_utf16_units(input: &[u16]) -> Option<Vec<u16>> {
    let mut output: Vec<u16> = Vec::with_capacity(200);
    let mut current: i32 = 0;
    let mut status = 0;

    for &ch in input {
        let c = (ch as i32) - 32;
        let s = status;
        status += 1;
        match s {
            0 => current = c << 1,
            15 => {
                output.push((current | c) as u16);
                status = 0;
            }
            k => {
                // Java cases 1..14: append (current | (c >> (15-k))); current = (c & (2^(15-k)-1)) << (k+1)
                let shift = 15 - k;
                output.push((current | (c >> shift)) as u16);
                current = (c & ((1 << shift) - 1)) << (k + 1);
            }
        }
    }

    decompress(&output)
}

/// `compressToUTF16` on a Rust string (UTF-16 in, UTF-16 out, converted at the edges).
pub fn compress_to_utf16(input: &str) -> String {
    let units: Vec<u16> = input.encode_utf16().collect();
    // The output alphabet is 32..=32+32767, which never contains a surrogate, so it is valid UTF-16.
    String::from_utf16(&compress_to_utf16_units(&units)).expect("compressToUTF16 output is BMP")
}

/// `decompressFromUTF16` on a Rust string; `None` on a malformed stream or non-UTF-16 result.
pub fn decompress_from_utf16(input: &str) -> Option<String> {
    let units: Vec<u16> = input.encode_utf16().collect();
    String::from_utf16(&decompress_from_utf16_units(&units)?).ok()
}

/// Java: `public static String decompressFromBase64(String input)`.
pub fn decompress_from_base64(input: &str) -> Option<Vec<u16>> {
    decompress(&decode64(input))
}

/// Java: `public static String decode64(String input)` (implemented from the JS version).
pub fn decode64(input: &str) -> Vec<u16> {
    let index_of = |b: u8| KEY_STR.iter().position(|&k| k == b).map(|p| p as i32).unwrap_or(-1);
    let bytes = input.as_bytes();
    let mut str_out: Vec<u16> = Vec::with_capacity(200);
    let mut ol = 0;
    let mut output_: i32 = 0;
    let mut i = 0;
    while i < bytes.len() {
        let at = |k: usize| bytes.get(k).copied().map(index_of).unwrap_or(-1);
        let enc1 = at(i);
        let enc2 = at(i + 1);
        let enc3 = at(i + 2);
        let enc4 = at(i + 3);
        i += 4;

        let chr1 = (enc1 << 2) | (enc2 >> 4);
        let chr2 = ((enc2 & 15) << 4) | (enc3 >> 2);
        let chr3 = ((enc3 & 3) << 6) | enc4;

        if ol % 2 == 0 {
            output_ = chr1 << 8;
            if enc3 != 64 {
                str_out.push((output_ | chr2) as u16);
            }
            if enc4 != 64 {
                output_ = chr3 << 8;
            }
        } else {
            str_out.push((output_ | chr1) as u16);
            if enc3 != 64 {
                output_ = chr2 << 8;
            }
            if enc4 != 64 {
                str_out.push((output_ | chr3) as u16);
            }
        }
        ol += 3;
    }
    str_out
}

/// Java: `public static String encode64(String input)`.
pub fn encode64(input: &[u16]) -> String {
    let key = |k: i32| KEY_STR[(k & 0x3f) as usize] as char;
    let byte_at = |i: usize| ((input[i >> 1] as i32) >> ((1 - (i & 1) as i32) << 3)) & 0xff;
    let mut result = String::with_capacity((input.len() * 8 + 1) / 3);
    let max = input.len() << 1;
    let mut i = 0;
    while i < max {
        let left = max - i;
        if left >= 3 {
            let ch1 = byte_at(i);
            let ch2 = byte_at(i + 1);
            let ch3 = byte_at(i + 2);
            i += 3;
            result.push(key(ch1 >> 2));
            result.push(key((ch1 << 4) + (ch2 >> 4)));
            result.push(key((ch2 << 2) + (ch3 >> 6)));
            result.push(key(ch3));
        } else if left == 2 {
            let ch1 = byte_at(i);
            let ch2 = byte_at(i + 1);
            i += 2;
            result.push(key(ch1 >> 2));
            result.push(key((ch1 << 4) + (ch2 >> 4)));
            result.push(key(ch2 << 2));
            result.push('=');
        } else {
            let ch1 = byte_at(i);
            i += 1;
            result.push(key(ch1 >> 2));
            result.push(key(ch1 << 4));
            result.push('=');
            result.push('=');
        }
    }
    result
}

/// Java: `public static String compressToBase64(String input)`.
pub fn compress_to_base64(input: &[u16]) -> String {
    encode64(&compress(input))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn units(s: &str) -> Vec<u16> {
        s.encode_utf16().collect()
    }

    /// Vectors produced by the Java class (`LZString.compressToUTF16`, ffb-common), 2026-11-23.
    const JAVA_VECTORS: &[(&str, &[u16])] = &[
        ("Lets see how much we can compress this string!",
         &[1634, 19499, 16828, 97, 4424, 736, 31523, 23744, 11681, 11814, 6218, 6125, 16483, 150,
           10706, 11936, 1824, 20032, 9900, 546, 11712, 17965, 26216, 3712, 4264, 32]),
        ("{\"netCommandId\":\"clientRequestVersion\"}",
         &[7137, 2109, 16748, 768, 24871, 22582, 24738, 3296, 2465, 4745, 124, 4152, 24621, 16491,
           1153, 8233, 8480, 18209, 11824, 1682, 16442, 16928, 20015, 2988, 2082, 31904, 32]),
        ("Ærø ünïcødé — ✓ 🏈 aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
         &[3201, 7207, 24616, 1040, 15143, 23608, 26144, 19497, 14528, 10276, 2617, 3681, 8344,
           7072, 10262, 4136, 13644, 31030, 10972, 20512, 32]),
        ("a", &[4328, 32]),
        ("", &[8224, 32]),
    ];

    #[test]
    fn compress_to_utf16_matches_java() {
        for (text, want) in JAVA_VECTORS {
            assert_eq!(compress_to_utf16_units(&units(text)), want.to_vec(), "input {:?}", text);
        }
    }

    #[test]
    fn decompress_from_utf16_matches_java() {
        for (text, wire) in JAVA_VECTORS {
            // Java: decompress of the empty stream's code returns "" via the end marker.
            assert_eq!(decompress_from_utf16_units(wire), Some(units(text)), "input {:?}", text);
        }
    }

    #[test]
    fn str_wrappers_round_trip() {
        let s = "{\"netCommandId\":\"serverModelSync\",\"commandNr\":17,\"x\":\"Ærø 🏈\"}";
        assert_eq!(decompress_from_utf16(&compress_to_utf16(s)).as_deref(), Some(s));
    }

    /// Large vectors (real game states; they hold coach names, so they stay outside git):
    /// `FFB_LZ_VECTOR_DIR` = a directory of `<name>` + `<name>.lz16` pairs written by the Java
    /// `LZString.compressToUTF16` (UTF-8 bytes, exactly one compressed WebSocket frame each).
    #[test]
    fn external_java_vectors() {
        let Ok(dir) = std::env::var("FFB_LZ_VECTOR_DIR") else { return };
        let mut n = 0;
        for e in std::fs::read_dir(&dir).unwrap() {
            let p = e.unwrap().path();
            if p.extension().map_or(false, |x| x == "lz16") {
                let wire = std::fs::read_to_string(&p).unwrap();
                let text = std::fs::read_to_string(p.with_extension("")).unwrap();
                assert_eq!(compress_to_utf16(&text), wire, "{}", p.display());
                assert_eq!(decompress_from_utf16(&wire).as_deref(), Some(text.as_str()), "{}", p.display());
                n += 1;
            }
        }
        assert!(n > 0, "no .lz16 vectors in {dir}");
    }

    #[test]
    fn decompress_empty_is_java_null() {
        assert_eq!(decompress(&[]), None);
    }

    #[test]
    fn base64_round_trip() {
        let u = units("Lets see how much we can compress this string!");
        assert_eq!(decompress_from_base64(&compress_to_base64(&u)), Some(u));
    }
}
