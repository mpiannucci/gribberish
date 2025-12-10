use bitflags::bitflags;
use std::collections::VecDeque;

use crate::error::GribberishError;

const ROS: u32 = 5;
/**
Size of the second extension table.
*/
const SE_TABLE_SIZE: usize = 90;

bitflags! {
    /// Represents a set of flags.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Flags: u8 {
        /**
        Whether the data is signed or not.
         */
        const AEC_DATA_SIGNED = 0b0000_0001;
        /**
        Whether to use 3-byte alignment (for data above 16 bits but below 24)
        */
        const AEC_DATA_3BYTE = 0b0000_0010;
        /**
        Whether to use MSB alignment.
        */
        const AEC_DATA_MSB = 0b0000_0100;
        /**
        Whether to use preprocessor.
        */
        const AEC_DATA_PREPROCESS = 0b0000_1000;
        /**
        Whether to use restricted set of code options.
        */
        const AEC_RESTRICTED =  0b0001_0000;
        /**
        Whether to pad RSI to byte boundary.
        */
        const AEC_PAD_RSI =  0b0010_0000;
        /**
        Whether to not enforce standard regarding legal block sizes.
        */
        const AEC_NOT_ENFORCE =  0b0100_0000;
        // The source may set any bits
        const _ = !0;
    }
}

/**
 * Implementation of the internal state of the CCSDS decoder. Heavily inspired by the libaec implementation,
 * and designed to work with ecmwf grib files.
 */
#[derive(Debug)]
pub struct InternalState {
    pub bits_per_sample: usize,
    pub block_size: u32,
    pub flags: Flags,
    pub next_in: VecDeque<u8>,
    pub next_out: Vec<u8>,
    pub avail_in: usize,
    pub avail_out: usize,
    pub total_in: usize,
    #[allow(dead_code)]
    pub total_out: usize,

    /** First not yet flushed byte in rsi_buffer */
    flush_start: usize,
    /** Bit pointer to the next unused bit in accumulator */
    bitp: usize,
    /** Accumulator for currently used bit sequence */
    acc: u64,
    /** Last fundamental sequence in accumulator */
    fs: u32,
    /** Option ID */
    id: u32,
    /** Bit length of code option identification key */
    id_len: usize,
    /** Table maps IDs to states */
    id_table: Vec<Mode>,
    /** Maximum integer for post-processing */
    xmax: u32,
    /** Minimum integer for post-processing */
    xmin: u32,
    /** True if postprocessor has to be used */
    pp: bool,
    /** Storage size of samples in bytes */
    bytes_per_sample: usize,
    /** Length of output block in bytes */
    out_blklen: usize,
    /** Length of uncompressed input block */
    in_blklen: usize,
    /** Block size minus reference sample if present */
    encoded_block_size: u32,

    offsets: Option<Vec<usize>>,
    sample_counter: u32,
    mode: Mode,

    /** Table for decoding second extension option */
    se_table: [i32; 2 * (SE_TABLE_SIZE + 1)],

    /** RSI in bytes */
    rsi_size: usize,
    reff: usize,
    /** Last output for post-processing */
    last_out: i32,

    /** Current position of output in rsi_buffer */
    rsip: usize,
    /** Reference sample interval, number of blocks between consecutive reference samples */
    rsi: u32,
    /** RSI buffer */
    rsi_buffer: Vec<u32>,
}

#[derive(Debug, Clone)]
enum Mode {
    LowEntropy,
    LowEntropyRef,
    ZeroBlock,
    ZeroOutput,
    SE,
    SEIncremental,
    Uncomp,
    Split,
    Id,
    NextCds,
    UncompCopy,
    SplitFs,
    SplitOutput,
}

enum DecodeStatus {
    Continue,
    Exit,
    Error(String),
}

impl InternalState {
    pub fn new(
        bits_per_sample: usize,
        block_size: u32,
        rsi: u32,
        flags: Flags,
        out_length: usize,
        data: Vec<u8>,
    ) -> Result<InternalState, String> {
        let next_out = Vec::new();
        let next_in = data.iter().copied().collect::<VecDeque<_>>();
        let avail_in = data.len();
        let avail_out = out_length;
        let total_in = 0;
        let total_out = 0;

        if bits_per_sample > 32 || bits_per_sample == 0 {
            return Err("Invalid bits_per_sample".into());
        }
        let (bytes_per_sample, id_len) = match bits_per_sample {
            25..=32 => (4, 5),
            17..=24 => (
                if flags.intersects(Flags::AEC_DATA_3BYTE) {
                    3
                } else {
                    4
                },
                5,
            ),
            9..=16 => (2, 4),
            5..=8 => (1, 3),
            4 => (1, 2),
            2..=3 => (1, 1),
            _ => return Err("Invalid bits_per_sample".into()),
        };
        let out_blklen = block_size * bytes_per_sample as u32;
        let (xmin, xmax) = if flags.intersects(Flags::AEC_DATA_SIGNED) {
            let xmax = ((1i64 << (bits_per_sample - 1)) - 1) as u32;
            (!xmax, xmax)
        } else {
            (0, ((1u64 << bits_per_sample) - 1) as u32)
        };
        let modi = (1u64 << id_len) as usize;
        let mut id_table = vec![Mode::Split; modi];
        id_table[0] = Mode::LowEntropy;
        id_table[modi - 1] = Mode::Uncomp;
        let rsi_size = rsi * block_size;
        let pp = flags.intersects(Flags::AEC_DATA_PREPROCESS);
        let reff: usize = if pp { 1 } else { 0 };
        let encoded_block_size: u32 = block_size - reff as u32;
        let rsi_buffer: Vec<u32> = vec![0; rsi_size as usize];
        let in_blklen: usize = ((block_size as usize) * bits_per_sample + id_len) / 8 + 16;

        Ok(Self {
            bits_per_sample,
            block_size,
            id_len,
            flags,
            next_in,
            next_out,
            avail_in,
            avail_out,
            total_in,
            total_out,
            rsi,
            rsip: 0,
            rsi_buffer,
            rsi_size: rsi_size as usize,
            flush_start: 0,
            bitp: 0,
            acc: 0,
            fs: 0,
            id: 0,
            reff,
            id_table,
            xmax,
            xmin,
            pp: flags.intersects(Flags::AEC_DATA_PREPROCESS),
            bytes_per_sample,
            out_blklen: out_blklen as usize,
            in_blklen,
            encoded_block_size,
            offsets: None,
            sample_counter: 0,
            mode: Mode::Id,
            se_table: create_se_table(),
            last_out: 0,
        })
    }

    fn run(&mut self) -> DecodeStatus {
        match self.mode {
            Mode::Id => self.run_id(),
            Mode::LowEntropy => self.run_low_entropy(),
            Mode::LowEntropyRef => self.run_low_entropy_ref(),
            Mode::ZeroBlock => self.run_zero_block(),
            Mode::ZeroOutput => self.run_zero_output(),
            Mode::Uncomp => self.run_uncomp(),
            Mode::UncompCopy => self.run_uncomp_copy(),
            Mode::SE => self.run_se(),
            Mode::SEIncremental => self.run_se_decode(),
            Mode::Split => self.run_split(),
            Mode::SplitFs => self.run_split_fs(),
            Mode::SplitOutput => self.run_split_output(),
            Mode::NextCds => self.run_next_cds(),
        }
    }

    fn put_bytes(&mut self, data: u32) {
        for i in 0..self.bytes_per_sample {
            if self.flags.intersects(Flags::AEC_DATA_MSB) {
                self.next_out
                    .push((data >> (8 * (self.bytes_per_sample - i - 1))) as u8);
            } else {
                self.next_out.push((data >> (8 * i)) as u8);
            }
        }
    }

    // Generic flush method that handles all byte orders
    pub fn flush_kind(&mut self) {
        let flush_end = self.rsip;

        if self.pp {
            // Handle preprocessed data
            if self.flush_start == 0 && self.rsip > 0 {
                self.last_out = self.rsi_buffer[0].try_into().unwrap();

                // Handle signed data
                if self.flags.intersects(Flags::AEC_DATA_SIGNED) {
                    let m = 1u32 << (self.bits_per_sample - 1);
                    let m2: i32 = m.try_into().unwrap();
                    self.last_out = (self.last_out ^ m2) - m2;
                }

                // byte_order.put_bytes(self.last_out.try_into().unwrap(), &mut self.next_out);
                self.put_bytes(self.last_out.try_into().unwrap());
                self.flush_start += 1;
            }

            let mut data: u32 = self.last_out as u32;
            let xmax: u32 = self.xmax as u32;

            if self.xmin == 0 {
                // Handle unsigned case
                let med = self.xmax / 2 + 1;

                for i in self.flush_start..flush_end {
                    let d = self.rsi_buffer[i];
                    let half_d = (d >> 1) + (d & 1);
                    let mask = if (data & med) == 0 { 0 } else { xmax };

                    if half_d <= (mask ^ data) {
                        // Use wrapping_sub for intentional bit manipulation:
                        // When d & 1 == 0: creates mask 0xFFFFFFFF
                        // When d & 1 == 1: creates mask 0x00000000
                        data = data.wrapping_add((d >> 1) ^ (!(d & 1).wrapping_sub(1)));
                    } else {
                        data = mask ^ d;
                    };

                    // byte_order.put_bytes(data, &mut self.next_out);
                    self.put_bytes(data);
                }
                self.last_out = data as i32;
            } else {
                // Handle signed case
                for i in self.flush_start..flush_end {
                    let d = self.rsi_buffer[i];
                    let half_d = (d >> 1) + (d & 1);
                    if (data as i32) < 0 {
                        if half_d <= xmax + data + 1 {
                            // Use wrapping operations for intentional bit manipulation
                            data = data.wrapping_add((d >> 1) ^ (!(d & 1).wrapping_sub(1)))
                        } else {
                            data = d - xmax - 1;
                        }
                    } else {
                        if half_d <= xmax - data {
                            // Use wrapping operations for intentional bit manipulation
                            data = data.wrapping_add((d >> 1) ^ (!(d & 1).wrapping_sub(1)))
                        } else {
                            data = xmax - d;
                        }
                    };

                    // byte_order.put_bytes(data, &mut self.next_out);
                    self.put_bytes(data);
                }
                self.last_out = data as i32;
            }
        } else {
            // Handle non-preprocessed data
            for i in self.flush_start..flush_end {
                let bp = self.rsi_buffer[i];
                // byte_order.put_bytes(bp, &mut self.next_out);
                self.put_bytes(bp);
            }
        }

        self.flush_start = self.rsip;
    }

    fn buffer_space(&self) -> bool {
        self.avail_in >= self.in_blklen && self.avail_out >= self.out_blklen
    }

    fn bits_get(&mut self, n: usize) -> u32 {
        return ((self.acc >> (self.bitp - n)) & (std::u64::MAX >> (64 - n))) as u32;
    }

    fn bits_ask(&mut self, n: usize) -> bool {
        while self.bitp < n {
            if !self.ask() {
                return false;
            }
        }
        true
    }

    fn bits_drop(&mut self, n: usize) {
        self.bitp -= n;
    }

    fn fs_drop(&mut self) {
        self.fs = 0;
        self.bitp -= 1;
    }

    fn ask(&mut self) -> bool {
        if self.avail_in == 0 {
            return false;
        }
        self.avail_in -= 1;
        let actual_next_in: u64 = self.next_in.pop_front().unwrap().into();
        self.acc <<= 8;
        self.acc |= actual_next_in;
        self.bitp += 8;
        true
    }

    fn fs_ask(&mut self) -> bool {
        if !self.bits_ask(1) {
            return false;
        }
        while (self.acc & (1u64 << (self.bitp - 1))) == 0 {
            if self.bitp == 1 {
                if !self.ask() {
                    return false;
                }
            }
            self.fs += 1;
            self.bitp -= 1;
        }
        true
    }

    fn rsi_used_size(&self) -> usize {
        self.rsip
    }

    fn copysample(&mut self) -> bool {
        if !self.bits_ask(self.bits_per_sample) || self.avail_out < self.bytes_per_sample {
            return false;
        }
        let sample = self.bits_get(self.bits_per_sample);
        self.put_sample(sample);
        self.bits_drop(self.bits_per_sample);
        true
    }

    fn put_sample(&mut self, sample: u32) {
        self.rsi_buffer[self.rsip] = sample;
        self.rsip += 1;
        self.avail_out -= self.bytes_per_sample;
    }

    fn put_sample_signed(&mut self, sample: i32) {
        self.rsi_buffer[self.rsip] = sample as u32;
        self.rsip += 1;
        self.avail_out -= self.bytes_per_sample;
    }

    fn direct_get_fs(&mut self) -> u32 {
        let mut fs: u32 = 0;
        if self.bitp > 0 {
            self.acc &= std::u64::MAX >> (64 - self.bitp);
        } else {
            self.acc = 0;
        }
        while self.acc == 0 {
            if self.avail_in < 7 {
                return 0;
            }
            self.direct_drain(7);
            fs += self.bitp as u32;
            self.bitp = 56;
        }
        let i = 63 - self.acc.leading_zeros() as usize;
        fs += (self.bitp - i - 1) as u32;
        self.bitp = i;
        return fs;
    }

    fn direct_drain(&mut self, b: usize) {
        let mut shift = b * 8;
        let mut acc = self.acc << shift;
        let drained = self.next_in.drain(..b);

        for byte in drained {
            shift -= 8;
            acc |= (byte as u64) << shift;
        }

        self.acc = acc;
        self.avail_in -= b;
    }

    pub fn direct_get(&mut self, n: usize) -> u32 {
        // Read n bits directly from input
        if self.bitp < n {
            let b = (63 - self.bitp) >> 3;

            self.direct_drain(b);

            self.bitp += b << 3;
        }
        self.bitp -= n as usize;

        ((self.acc >> self.bitp) & (std::u64::MAX >> (64 - n as u64))) as u32
    }

    fn run_id(&mut self) -> DecodeStatus {
        if self.avail_in >= self.in_blklen.try_into().unwrap() {
            self.id = self.direct_get(self.id_len);
        } else {
            if !self.bits_ask(self.id_len) {
                self.mode = Mode::Id;
                return DecodeStatus::Exit;
            }
            self.id = self.bits_get(self.id_len);
            self.bits_drop(self.id_len);
        }

        self.mode = self.id_table[self.id as usize].clone();
        DecodeStatus::Continue
    }

    fn run_low_entropy_ref(&mut self) -> DecodeStatus {
        if self.reff != 0 && !self.copysample() {
            return DecodeStatus::Exit;
        }

        if self.id == 1 {
            self.mode = Mode::SE;
        } else {
            self.mode = Mode::ZeroBlock;
        }
        DecodeStatus::Continue
    }

    fn run_low_entropy(&mut self) -> DecodeStatus {
        if !self.bits_ask(1) {
            return DecodeStatus::Exit;
        }
        self.id = self.bits_get(1);
        self.bits_drop(1);
        self.mode = Mode::LowEntropyRef;
        DecodeStatus::Continue
    }

    fn run_zero_output(&mut self) -> DecodeStatus {
        loop {
            if self.avail_out < self.bytes_per_sample {
                return DecodeStatus::Exit;
            }
            self.put_sample(0);
            self.sample_counter -= 1;
            if self.sample_counter <= 0 {
                break;
            }
        }
        self.mode = Mode::NextCds;
        DecodeStatus::Continue
    }

    fn run_zero_block(&mut self) -> DecodeStatus {
        if !self.fs_ask() {
            return DecodeStatus::Exit;
        }
        let mut zero_blocks: u32 = self.fs + 1;
        self.fs_drop();

        if zero_blocks == ROS {
            let b = (self.rsi_used_size() as i32) / self.block_size as i32;
            zero_blocks = std::cmp::min(self.rsi as i32 - b, 64 - (b % 64)) as u32;
        } else if zero_blocks > ROS {
            zero_blocks -= 1;
        }

        let reff = self.reff as u32;

        let zero_samples = (zero_blocks * self.block_size - reff) as usize;
        if (self.rsi_size - self.rsi_used_size()) < zero_samples {
            return DecodeStatus::Error(format!(
                "Not enough space to write zero samples: size {} used {} needed {} blocks: {}",
                self.rsi_size,
                self.rsi_used_size(),
                zero_samples,
                zero_blocks
            ));
        }
        let zero_bytes = (zero_samples as usize) * self.bytes_per_sample;
        if self.avail_out >= zero_bytes {
            for _ in 0..zero_samples {
                self.rsi_buffer[self.rsip] = 0;
                self.rsip += 1;
                self.avail_out -= self.bytes_per_sample;
            }
            self.mode = Mode::NextCds;
        } else {
            self.sample_counter = zero_samples as u32;
            self.mode = Mode::ZeroOutput;
        }
        DecodeStatus::Continue
    }

    fn run_se(&mut self) -> DecodeStatus {
        if self.buffer_space() {
            // We have enough output buffer space
            let mut i: u32 = self.reff as u32;

            while i < self.block_size {
                // Get the next value from input stream
                let m = self.direct_get_fs();

                // Validate m is within bounds
                if m > SE_TABLE_SIZE as u32 {
                    return DecodeStatus::Error(format!("SE table index out of bounds (se) {}", m));
                }
                let d1: i32 = (m as i32) - self.se_table[(2 * m + 1) as usize];

                // Handle even-numbered samples
                if (i & 1) == 0 {
                    self.put_sample_signed(self.se_table[(2 * m) as usize] - d1);
                    i += 1;
                }

                // Handle all samples
                self.put_sample_signed(d1);
                i += 1;
            }

            self.mode = Mode::NextCds;
        } else {
            // Not enough output buffer space, switch to incremental processing
            self.sample_counter = self.reff as u32;
            self.mode = Mode::SEIncremental;
        }

        DecodeStatus::Continue
    }

    fn run_se_decode(&mut self) -> DecodeStatus {
        while self.sample_counter < self.block_size {
            // Get next value from input stream
            if !self.fs_ask() {
                return DecodeStatus::Exit;
            }

            let m: i32 = self.fs as i32;

            // Validate m is within bounds
            if m > SE_TABLE_SIZE.try_into().unwrap() {
                return DecodeStatus::Error(format!(
                    "SE table index out of bounds (se_decode) {}",
                    m
                ));
            }

            let d1: i32 = m - self.se_table[(2 * m + 1) as usize];

            // Handle even-numbered samples
            if (self.sample_counter & 1) == 0 {
                if self.avail_out < self.bytes_per_sample {
                    return DecodeStatus::Exit;
                }
                self.put_sample_signed(self.se_table[(2 * m) as usize] - d1);
                self.sample_counter += 1;
            }

            // Handle all samples
            if self.avail_out < self.bytes_per_sample {
                return DecodeStatus::Exit;
            }
            self.put_sample_signed(d1);
            self.sample_counter += 1;
            self.fs_drop();
        }

        self.mode = Mode::NextCds;
        DecodeStatus::Continue
    }

    fn run_uncomp(&mut self) -> DecodeStatus {
        if self.buffer_space() {
            // We have enough output buffer space to process the entire block at once
            for _ in 0..self.block_size {
                self.rsi_buffer[self.rsip] = self.direct_get(self.bits_per_sample);
                self.rsip += 1;
            }
            self.avail_out -= self.out_blklen;
            self.mode = Mode::NextCds;
        } else {
            // Not enough output space, switch to incremental processing
            self.sample_counter = self.block_size;
            self.mode = Mode::UncompCopy;
        }
        DecodeStatus::Continue
    }

    // Add this method for incremental processing
    fn run_uncomp_copy(&mut self) -> DecodeStatus {
        loop {
            if !self.copysample() {
                return DecodeStatus::Exit;
            }
            self.sample_counter -= 1;
            if self.sample_counter == 0 {
                break;
            }
        }

        self.mode = Mode::NextCds;
        DecodeStatus::Continue
    }

    fn run_split(&mut self) -> DecodeStatus {
        if self.buffer_space() {
            // Process entire block at once when we have enough buffer space
            let k: i32 = (self.id as i32) - 1;
            let binary_part = ((k as usize) * self.encoded_block_size as usize) / 8 + 9;

            // Handle reference sample if needed
            if self.reff != 0 {
                self.rsi_buffer[self.rsip] = self.direct_get(self.bits_per_sample);
                self.rsip += 1;
            }

            // First pass: get fundamental sequence values
            for i in 0..self.encoded_block_size {
                self.rsi_buffer[self.rsip + i as usize] = self.direct_get_fs() << k;
            }

            // Second pass: add remainder bits if k > 0
            if k != 0 {
                if self.avail_in < binary_part {
                    return DecodeStatus::Error("Insufficient input for binary part".into());
                }

                for _ in 0..self.encoded_block_size {
                    self.rsi_buffer[self.rsip] += self.direct_get(k as usize);
                    self.rsip += 1;
                }
            } else {
                // No remainder bits, just copy base values
                self.rsip += self.encoded_block_size as usize;
            }

            self.avail_out -= self.out_blklen as usize;
            self.mode = Mode::NextCds;
        } else {
            if (self.reff != 0) && !self.copysample() {
                return DecodeStatus::Exit;
            }
            self.sample_counter = 0;
            self.mode = Mode::SplitFs;
        }

        DecodeStatus::Continue
    }

    fn run_split_fs(&mut self) -> DecodeStatus {
        let k = self.id - 1;

        loop {
            // Get fundamental sequence value
            if !self.fs_ask() {
                return DecodeStatus::Exit;
            }
            // Store base value
            self.rsi_buffer[self.rsip + self.sample_counter as usize] = self.fs << k;
            self.fs_drop();
            self.sample_counter += 1;
            // Break when we've processed all samples in the block
            if self.sample_counter >= self.encoded_block_size {
                break;
            }
        }

        self.sample_counter = 0;
        self.mode = Mode::SplitOutput;
        DecodeStatus::Continue
    }

    fn run_split_output(&mut self) -> DecodeStatus {
        let k = self.id - 1;
        loop {
            // Check if we have enough output space
            if !self.bits_ask(k as usize) || self.avail_out < self.bytes_per_sample {
                return DecodeStatus::Exit;
            }

            // Get remainder bits if k > 0
            if k != 0 {
                self.rsi_buffer[self.rsip] += self.bits_get(k as usize);
                self.rsip += 1;
            } else {
                self.rsip += 1;
            }
            self.avail_out -= self.bytes_per_sample;
            self.bits_drop(k as usize);
            self.sample_counter += 1;
            // Break when we've processed all samples in the block
            if self.sample_counter >= self.encoded_block_size {
                break;
            }
        }

        self.mode = Mode::NextCds;
        DecodeStatus::Continue
    }

    fn run_next_cds(&mut self) -> DecodeStatus {
        // If we're tracking offsets and we've reached the RSI size
        if let Some(offsets) = &mut self.offsets {
            if self.rsi_buffer.len() == self.block_size as usize {
                // Calculate and store bit offset
                let bit_offset = self.total_in * 8 - (self.avail_in * 8 + self.bitp);
                offsets.push(bit_offset);
            }
        }

        // Check if we've reached the RSI size
        if self.rsi_size == self.rsi_used_size() {
            // Flush output and reset buffers
            self.flush_kind();
            self.flush_start = 0;
            self.rsip = 0;

            // Handle preprocessing flag
            if self.pp {
                self.reff = 1;
                self.encoded_block_size = self.block_size - 1;
            }

            // Handle RSI padding
            if self.flags.intersects(Flags::AEC_PAD_RSI) {
                self.bitp -= self.bitp % 8;
            }
        } else {
            // Not at RSI boundary, prepare for next block
            self.reff = 0;
            self.encoded_block_size = self.block_size;
        }

        // Switch back to ID mode
        return self.run_id();
    }
}

fn read_u32_from_bytes(bytes: &[u8], bytes_per_sample: usize) -> Vec<f32> {
    if bytes_per_sample == 0 || bytes_per_sample > 4 {
        return Vec::new();
    }

    match bytes_per_sample {
        1 => bytes.iter().map(|&b| b as f32).collect(),
        2 => bytes
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes(chunk.try_into().unwrap()) as f32)
            .collect(),
        3 => bytes
            .chunks_exact(3)
            .map(|chunk| {
                // Pad the 3-byte chunk to 4 bytes for u32 conversion
                let mut buf = [0u8; 4];
                buf[..3].copy_from_slice(chunk);
                u32::from_le_bytes(buf) as f32
            })
            .collect(),
        4 => bytes
            .chunks_exact(4)
            .map(|chunk| f32::from_le_bytes(chunk.try_into().unwrap()))
            .collect(),
        _ => Vec::new(),
    }
}

fn create_se_table() -> [i32; 2 * (SE_TABLE_SIZE + 1)] {
    let mut table = [0; 2 * (SE_TABLE_SIZE + 1)];
    let mut k: i32 = 0;
    for i in 0..13 {
        let ms: i32 = k;
        for _ in 0..=i {
            let ksize = k as usize;
            table[2 * ksize] = i;
            table[2 * ksize + 1] = ms;
            k += 1;
        }
    }
    table
}

fn is_big_endian() -> bool {
    return cfg!(target_endian = "big");
}

fn modify_aec_flags(flags: Flags) -> Flags {
    let mut new_flags = flags;
    new_flags &= !Flags::AEC_DATA_3BYTE; // disable support for 3-bytes per value
    if is_big_endian() {
        new_flags |= Flags::AEC_DATA_MSB; // enable big-endian
    } else {
        new_flags &= !Flags::AEC_DATA_MSB; // enable little-endian
    }
    new_flags
}

pub fn extract_ccsds_data(
    data: Vec<u8>,
    block_len: u8,
    compression_options_mask: u8,
    avail_out: usize,
    reference_sample_interval: u16,
    bits_per_sample: usize,
) -> Result<Vec<f32>, GribberishError> {
    let nbytes_per_sample: usize = (bits_per_sample + 7) / 8;

    let flags = modify_aec_flags(Flags::from_bits_truncate(compression_options_mask));

    // Prepare the input stream
    let state_or_error = InternalState::new(
        bits_per_sample,
        block_len as u32,
        reference_sample_interval as u32,
        flags,
        avail_out,
        data,
    );
    if let Err(e) = state_or_error {
        return Err(GribberishError::MessageError(e.to_string()));
    }
    let mut state = state_or_error.unwrap();

    // Decode the data
    let mut status: DecodeStatus;
    let mut count = 0;
    loop {
        status = state.run();
        count += 1;

        match status {
            DecodeStatus::Continue => continue,
            DecodeStatus::Exit => break,
            DecodeStatus::Error(msg) => {
                return Err(GribberishError::MessageError(format!(
                    "Error: {:?} at count: {:?}, still available: {:?}, processed: {:?}",
                    msg,
                    count,
                    state.avail_out,
                    state.next_out.len()
                )))
            }
        }
    }

    // Validate
    if matches!(status, DecodeStatus::Exit)
        && state.avail_out > 0
        && state.avail_out < state.bytes_per_sample
    {
        return Err(GribberishError::MessageError("Memory error".to_string()));
    }

    // Flush remaining data
    state.flush_kind();
    let decompressed_data: Vec<f32> =
        read_u32_from_bytes(state.next_out.as_slice(), nbytes_per_sample);
    Ok(decompressed_data)
}
