#[derive(Clone)]

//逆向于https://open.spotifycdn.com/cdn/build/web-player/web-player.1234abcd.js
pub struct Sha1 {
    state:     [u32; 5],
    buffer:    [u8; 64],
    buf_len:   usize,
    total_len: u64,
    finished:  bool,
}

impl Sha1 {
    pub fn new() -> Self {
        Self {
            state:     [0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476, 0xc3d2e1f0],
            buffer:    [0u8; 64],
            buf_len:   0,
            total_len: 0,
            finished:  false,
        }
    }

    pub fn update(&mut self, data: &[u8]) {
        assert!(!self.finished);
        self.total_len += data.len() as u64;
        let mut off = 0usize;

        if self.buf_len > 0 {
            let take = (64 - self.buf_len).min(data.len());
            self.buffer[self.buf_len..self.buf_len + take].copy_from_slice(&data[..take]);
            self.buf_len += take;
            off += take;
            if self.buf_len == 64 {
                let b = self.buffer;
                Self::compress(&mut self.state, &b);
                self.buf_len = 0;
            }
        }
        while off + 64 <= data.len() {
            let block: [u8; 64] = data[off..off + 64].try_into().unwrap();
            Self::compress(&mut self.state, &block);
            off += 64;
        }
        let rem = data.len() - off;
        if rem > 0 {
            self.buffer[..rem].copy_from_slice(&data[off..]);
            self.buf_len = rem;
        }
    }

    pub fn digest_into(mut self, out: &mut [u8; 20]) {
        self.finished = true;
        let bit_len = self.total_len * 8;

        self.buffer[self.buf_len] = 0x80;
        for b in self.buffer[self.buf_len + 1..].iter_mut() { *b = 0; }

        if self.buf_len + 1 > 56 {
            let b = self.buffer;
            Self::compress(&mut self.state, &b);
            self.buffer = [0u8; 64];
        }
        self.buffer[56..64].copy_from_slice(&bit_len.to_be_bytes());
        let b = self.buffer;
        Self::compress(&mut self.state, &b);

        for (i, &w) in self.state.iter().enumerate() {
            out[i * 4..i * 4 + 4].copy_from_slice(&w.to_be_bytes());
        }
    }

    pub fn digest(self) -> [u8; 20] {
        let mut out = [0u8; 20];
        self.digest_into(&mut out);
        out
    }

    fn compress(state: &mut [u32; 5], block: &[u8; 64]) {
        let mut w = [0u32; 80];
        for i in 0..16 {
            w[i] = u32::from_be_bytes(block[i * 4..i * 4 + 4].try_into().unwrap());
        }
        for i in 16..80 {
            w[i] = (w[i-3] ^ w[i-8] ^ w[i-14] ^ w[i-16]).rotate_left(1);
        }
        let [mut a, mut b, mut c, mut d, mut e] = *state;
        for i in 0..80 {
            let (f, k): (u32, u32) = match i {
                0..=19  => ((b & c) | (!b & d),           0x5a827999),
                20..=39 => (b ^ c ^ d,                    0x6ed9eba1),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8f1bbcdc),
                _       => (b ^ c ^ d,                    0xca62c1d6),
            };
            let t = a.rotate_left(5).wrapping_add(f).wrapping_add(e)
                     .wrapping_add(k).wrapping_add(w[i]);
            e = d; d = c; c = b.rotate_left(30); b = a; a = t;
        }
        state[0] = state[0].wrapping_add(a);
        state[1] = state[1].wrapping_add(b);
        state[2] = state[2].wrapping_add(c);
        state[3] = state[3].wrapping_add(d);
        state[4] = state[4].wrapping_add(e);
    }
}

//本质class S
pub struct HmacSha1 {
    i_hash: Sha1,
    o_hash: Sha1,
}

impl HmacSha1 {
    pub fn new(key: &[u8]) -> Self {
        let mut k = [0u8; 64];
        k[..key.len().min(64)].copy_from_slice(&key[..key.len().min(64)]);

        let mut ipad = k; for b in ipad.iter_mut() { *b ^= 0x36; }
        let mut opad = k; for b in opad.iter_mut() { *b ^= 0x5c; }

        let mut i_hash = Sha1::new(); i_hash.update(&ipad);
        let mut o_hash = Sha1::new(); o_hash.update(&opad);
        Self { i_hash, o_hash }
    }

    pub fn update(&mut self, data: &[u8]) -> &mut Self {
        self.i_hash.update(data); self
    }

    pub fn digest(mut self) -> [u8; 20] {
        let inner = self.i_hash.digest();
        self.o_hash.update(&inner);
        self.o_hash.digest()
    }

    pub fn oneshot(key: &[u8], data: &[u8]) -> [u8; 20] {
        let mut h = Self::new(key); h.update(data); h.digest()
    }
}


fn counter_to_bytes(counter: u64) -> [u8; 8] {
    counter.to_be_bytes()
}


pub fn hotp(secret: &[u8], counter: u64, digits: u32) -> String {
    let hmac   = HmacSha1::oneshot(secret, &counter_to_bytes(counter));
    let offset = (hmac[19] & 0x0f) as usize;
    let code   = ((hmac[offset]     as u32 & 0x7f) << 24)
               | ((hmac[offset + 1] as u32 & 0xff) << 16)
               | ((hmac[offset + 2] as u32 & 0xff) << 8)
               |  (hmac[offset + 3] as u32 & 0xff);
    format!("{:0>width$}", code % 10u32.pow(digits), width = digits as usize)
}

// 本质e6
pub struct Totp {
    pub secret:  Vec<u8>,
    pub period:  u64,
    pub digits:  u32,
    pub version: u32,
}

impl Totp {
    pub fn new(secret: Vec<u8>, period: u64, digits: u32, version: u32) -> Self {
        Self { secret, period, digits, version }
    }

    /// e6.counter({ period, timestamp })
    pub fn counter(&self, timestamp_ms: u64) -> u64 {
        timestamp_ms / 1000 / self.period
    }

    /// e6.generate({ timestamp })
    pub fn generate(&self, timestamp_ms: u64) -> String {
        hotp(&self.secret, self.counter(timestamp_ms), self.digits)
    }

    /// to(Date.now())
    pub fn generate_now(&self) -> String {
        let ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap().as_millis() as u64;
        self.generate(ms)
    }
}



pub struct TotpPayload {
    pub reason:       String,
    pub product_type: String,
    pub totp:         String,
    pub totp_server:  String,
    pub totp_ver:     String,
}

//本质tl函数
pub fn tl(ts: &Totp, reason: &str, product_type: &str, server_ts_s: Option<u64>) -> TotpPayload {
    let totp        = ts.generate_now();
    let totp_server = match server_ts_s {
        Some(s) => ts.generate(s * 1000),   // 1e3 * r
        None    => "unavailable".to_string(),
    };
    TotpPayload {
        reason:       reason.to_string(),
        product_type: product_type.to_string(),
        totp,
        totp_server,
        totp_ver: ts.version.to_string(),
    }
}

//这些是把处理后的数据硬编码好
pub fn build_totp(index: usize) -> Totp {
    let (bytes, version): (&[u8], u32) = match index {
        // version 61  (60 bytes)
        0 => (&[
            51,55,54,49,51,54,51,56,55,53,51,56,52,53,57,56,57,51,56,56,
            51,51,49,50,51,49,48,57,49,49,57,57,50,56,52,55,49,49,50,52,
            52,56,56,57,52,52,49,48,50,49,48,53,49,49,50,57,55,49,48,56,
        ], 61),
        // version 60  (46 bytes)
        1 => (&[
            55,48,49,48,51,55,56,49,49,57,56,55,55,57,51,51,57,48,55,57,
            52,56,52,49,51,54,56,51,56,49,55,53,55,55,57,57,51,55,54,52,
            57,50,55,52,55,51,
        ], 60),
        // version 59  (70 bytes)
        2 => (&[
            49,49,52,57,57,54,56,55,52,57,57,53,51,53,57,49,48,57,52,53,
            51,53,54,55,56,50,55,54,57,51,55,49,55,56,51,56,52,55,57,54,
            53,55,49,48,52,52,55,52,51,49,50,53,49,48,56,50,56,49,50,49,
            49,52,50,49,55,56,57,57,57,54,
        ], 59),
        _ => panic!("index out of range, valid: 0/1/2"),
    };
    Totp::new(bytes.to_vec(), 30, 6, version)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hotp_rfc4226_case0() {
        assert_eq!(hotp(b"12345678901234567890", 0, 6), "755224");
    }

    #[test]
    fn hotp_rfc4226_case1() {
        assert_eq!(hotp(b"12345678901234567890", 1, 6), "287082");
    }

    #[test]
    fn totp_rfc6238_t59() {
        let ts = Totp::new(b"12345678901234567890".to_vec(), 30, 6, 0);
        assert_eq!(ts.generate(59_000), "287082");
    }

    #[test]
    fn counter_bytes_endian() {
        // JS: i(59374914) = [0,0,0,0,3,136,173,66]
        assert_eq!(counter_to_bytes(59_374_914), [0, 0, 0, 0, 3, 136, 173, 66]);
    }
}
