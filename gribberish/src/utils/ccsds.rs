use bitflags::bitflags;
use std::{collections::VecDeque, io::Read};

use crate::error::GribberishError;

const ROS: usize = 5;
const SE_TABLE_SIZE: u32 = 90;

/*
Samples are signed. Telling libaec this results in a slightly
 * better compression ratio. Default is unsigned.
#define AEC_DATA_SIGNED 1

/* 24 bit samples are coded in 3 bytes */
#define AEC_DATA_3BYTE 2

/* Samples are stored with their most significant bit first. This has
 * nothing to do with the endianness of the host. Default is LSB. */
#define AEC_DATA_MSB 4

/* Set if preprocessor should be used */
#define AEC_DATA_PREPROCESS 8

/* Use restricted set of code options */
#define AEC_RESTRICTED 16

/* Pad RSI to byte boundary. Only used for decoding some CCSDS sample
 * data. Do not use this to produce new data as it violates the
 * standard. */
#define AEC_PAD_RSI 32

/* Do not enforce standard regarding legal block sizes. */
#define AEC_NOT_ENFORCE 64


14 => 24
AEC_DATA_3BYTE + AEC_DATA_MSB + AEC_DATA_PREPROCESS
 */

bitflags! {
    /// Represents a set of flags.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Flags: u8 {
        const AEC_DATA_SIGNED = 0b0000_0001;
        const AEC_DATA_3BYTE = 0b0000_0010;
        const AEC_DATA_MSB = 0b0000_0100;
        const AEC_DATA_PREPROCESS = 0b0000_1000;
        const AEC_RESTRICTED =  0b0001_0000;
        const AEC_PAD_RSI =  0b0010_0000;
        const AEC_NOT_ENFORCE =  0b0100_0000;
        // The source may set any bits
        const _ = !0;
    }
}

#[derive(Debug)]
pub struct InternalState {
    pub bits_per_sample: usize,
    pub block_size: usize,
    pub flags: Flags,
    pub next_in: VecDeque<u8>,
    pub next_out: Vec<u8>,
    pub avail_in: usize,
    pub avail_out: usize,
    pub total_in: usize,
    pub total_out: usize,

    // pub decompressed_data: Vec<u8>,
    /** First not yet flushed byte in rsi_buffer */
    flush_start: usize,
    /** Bit pointer to the next unused bit in accumulator */
    bitp: usize,
    /** Accumulator for currently used bit sequence */
    acc: u64,
    /** Last fundamental sequence in accumulator */
    fs: u32,
    /** Option ID */
    id: usize,
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
    out_blklen: u32,
    /** Length of uncompressed input block */
    in_blklen: u32,
    /** Block size minus reference sample if present */
    encoded_block_size: u32,

    offsets: Option<Vec<usize>>,
    sample_counter: u32,
    mode: Mode,

    /** Table for decoding second extension option */
    se_table: Vec<i32>,

    /** RSI in bytes */
    rsi_size: usize,
    reff: usize,
    /** Last output for post-processing */
    last_out: i32,
    
    /** Current position of output in rsi_buffer */
    rsip: usize,
    rsi: usize,
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

// First, define an enum for the different byte orders
#[derive(Debug, Clone, Copy)]
pub enum ByteOrder {
    MSB32,
    MSB24,
    MSB16,
    LSB32,
    LSB24,
    LSB16,
    Byte8,
}

impl ByteOrder {
    // Helper method to write bytes in the correct order
    fn put_bytes(&self, data: u32, output: &mut Vec<u8>) {
        match self {
            ByteOrder::MSB32 => {
                output.push((data >> 24) as u8);
                output.push((data >> 16) as u8);
                output.push((data >> 8) as u8);
                output.push(data as u8);
            }
            ByteOrder::MSB24 => {
                output.push((data >> 16) as u8);
                output.push((data >> 8) as u8);
                output.push(data as u8);
            }
            ByteOrder::MSB16 => {
                output.push((data >> 8) as u8);
                output.push(data as u8);
            }
            ByteOrder::LSB32 => {
                output.push(data as u8);
                output.push((data >> 8) as u8);
                output.push((data >> 16) as u8);
                output.push((data >> 24) as u8);
            }
            ByteOrder::LSB24 => {
                output.push(data as u8);
                output.push((data >> 8) as u8);
                output.push((data >> 16) as u8);
            }
            ByteOrder::LSB16 => {
                output.push(data as u8);
                output.push((data >> 8) as u8);
            }
            ByteOrder::Byte8 => {
                output.push(data as u8);
            }
        }
    }
}

impl InternalState {
    pub fn new(
        bits_per_sample: usize,
        block_size: usize,
        rsi: usize,
        flags: Flags,
        out_length: usize,
        data: Vec<u8>,
    ) -> Result<InternalState, String> {
        // Self {
        //     bits_per_sample,
        //     block_size,
        //     flags,
        //     next_in: VecDeque::new(),
        //     next_out: Vec::new(),
        //     avail_in: 0,
        //     avail_out: 0,
        //     total_in: 0,
        //     total_out: 0
        // }
        // let next_in = VecDeque::new();
        let next_out = Vec::new();
        let next_in = data.iter().copied().collect::<VecDeque<_>>();
        let avail_in = data.len();
        let avail_out = out_length;
        let total_in = 0;
        let total_out = 0;

        if bits_per_sample > 32 || bits_per_sample == 0 {
            return Err("Invalid bits_per_sample".into());
        }

        // let bytes_per_sample = if bits_per_sample > 16 {
        //     if bits_per_sample <= 24 && flags.intersects(Flags::AEC_DATA_3BYTE) {
        //         3
        //     } else {
        //         4
        //     }
        // } else if bits_per_sample > 8 {
        //     2
        // } else {
        //     1
        // };

        /*

        if (strm->bits_per_sample > 16) {
            state->id_len = 5;

            if (strm->bits_per_sample <= 24 && strm->flags & AEC_DATA_3BYTE) {
                state->bytes_per_sample = 3;
                if (strm->flags & AEC_DATA_MSB)
                    state->flush_output = flush_msb_24;
                else
                    state->flush_output = flush_lsb_24;
            } else {
                state->bytes_per_sample = 4;
                if (strm->flags & AEC_DATA_MSB)
                    state->flush_output = flush_msb_32;
                else
                    state->flush_output = flush_lsb_32;
            }
            state->out_blklen = strm->block_size * state->bytes_per_sample;
        }
        else if (strm->bits_per_sample > 8) {
            state->bytes_per_sample = 2;
            state->id_len = 4;
            state->out_blklen = strm->block_size * 2;
            if (strm->flags & AEC_DATA_MSB)
                state->flush_output = flush_msb_16;
            else
                state->flush_output = flush_lsb_16;
        } else {
            if (strm->flags & AEC_RESTRICTED) {
                if (strm->bits_per_sample <= 4) {
                    if (strm->bits_per_sample <= 2)
                        state->id_len = 1;
                    else
                        state->id_len = 2;
                } else {
                    return AEC_CONF_ERROR;
                }
            } else {
                state->id_len = 3;
            }

            state->bytes_per_sample = 1;
            state->out_blklen = strm->block_size;
            state->flush_output = flush_8;
        }
         */
        let bytes_per_sample;
        let id_len;
        match bits_per_sample {
            25..=32 => {
                id_len = 5;
                bytes_per_sample = 4;
            }
            17..=24 => {
                id_len = 5;
                if flags.intersects(Flags::AEC_DATA_3BYTE) {
                    bytes_per_sample = 3;
                } else {
                    bytes_per_sample = 4;
                }
            }
            9..=16 => {
                id_len = 4;
                bytes_per_sample = 2;
            }
            5..=8 => {
                bytes_per_sample = 1;
                id_len = 3;
            }
            4 => {
                bytes_per_sample = 1;
                id_len = 2;
            }
            2..=3 => {
                bytes_per_sample = 1;
                id_len = 1;
            }
            _ => {
                return Err("Invalid bits_per_sample".into());
            }
        }
        let out_blklen: u32 = (block_size * bytes_per_sample) as u32;

        /*
                 if (strm->flags & AEC_DATA_SIGNED) {
            state->xmax = (INT64_C(1) << (strm->bits_per_sample - 1)) - 1;
            state->xmin = ~state->xmax;
        } else {
            state->xmin = 0;
            state->xmax = (UINT64_C(1) << strm->bits_per_sample) - 1;
        }
             */

        let xmax: u32;
        let xmin: u32;
        if flags.intersects(Flags::AEC_DATA_SIGNED) {
            xmax = (1 << (bits_per_sample - 1)) - 1;
            xmin = !(xmax);
        } else {
            xmin = 0;
            xmax = (1 << bits_per_sample) - 1;
        }

        /*
                modi = 1UL << state->id_len;
        state->id_table = malloc(modi * sizeof(int (*)(struct aec_stream *)));
        if (state->id_table == NULL)
            return AEC_MEM_ERROR;

        state->id_table[0] = m_low_entropy;
        for (int i = 1; i < modi - 1; i++) {
            state->id_table[i] = m_split;
        }
        state->id_table[modi - 1] = m_uncomp;

        state->rsi_size = strm->rsi * strm->block_size;
        state->rsi_buffer = malloc(state->rsi_size * sizeof(uint32_t));
        if (state->rsi_buffer == NULL)
            return AEC_MEM_ERROR;

            */

        let modi = 1 << id_len;
        let mut id_table = vec![Mode::LowEntropy; modi as usize];
        for i in 1..modi as usize - 1 {
            id_table[i] = Mode::Split;
        }
        id_table[modi as usize - 1] = Mode::Uncomp;

        /*
                state->rsi_size = strm->rsi * strm->block_size;
        state->rsi_buffer = malloc(state->rsi_size * sizeof(uint32_t));
        if (state->rsi_buffer == NULL)
            return AEC_MEM_ERROR;

        state->pp = strm->flags & AEC_DATA_PREPROCESS;
        if (state->pp) {
            state->ref = 1;
            state->encoded_block_size = strm->block_size - 1;
        } else {
            state->ref = 0;
            state->encoded_block_size = strm->block_size;
        }
        strm->total_in = 0;
        strm->total_out = 0;

        state->rsip = state->rsi_buffer;
        state->flush_start = state->rsi_buffer;
        state->bitp = 0;
        state->fs = 0;
        state->mode = m_id;
        state->offsets = NULL;
         */
        let rsi_size = rsi * block_size;
        let pp = flags.intersects(Flags::AEC_DATA_PREPROCESS);
        let reff;
        let encoded_block_size: u32;
        if pp {
            reff = 1;
            encoded_block_size = (block_size - 1) as u32;
        } else {
            reff = 0;
            encoded_block_size = block_size as u32;
        }
        
        let rsi_buffer: Vec<u32> = vec![0; rsi_size as usize];

        /*
            state->in_blklen = (strm->block_size * strm->bits_per_sample
                        + state->id_len) / 8 + 16;
                         */
        let in_blklen: u32 = ((block_size * bits_per_sample + id_len) / 8 + 16) as u32;

        Ok(Self {
            bits_per_sample,
            block_size,
            id_len,
            flags: flags,
            next_in,
            next_out,
            avail_in,
            avail_out,
            total_in,
            total_out,
            rsi,
            rsip: 0,
            rsi_buffer,
            rsi_size,
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
            out_blklen,
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
            Mode::ZeroBlock => self.run_zero_block(),
            Mode::ZeroOutput => self.run_zero_output(),
            Mode::SE => self.run_se(),
            Mode::SEIncremental => self.run_se_decode(),
            Mode::Uncomp => self.run_uncomp(),
            Mode::Split => self.run_split(),
            Mode::NextCds => self.run_next_cds(),
            Mode::UncompCopy => self.run_uncomp_copy(),
            Mode::SplitFs => self.run_split_fs(),
            Mode::SplitOutput => self.run_split_output(),
            Mode::LowEntropyRef => self.run_low_entropy_ref(),
        }
    }

    /*

    #define FLUSH(KIND)                                                      \
    static void flush_##KIND(struct aec_stream *strm)                    \
    {                                                                    \
        uint32_t *flush_end, *bp, half_d;                                \
        uint32_t xmax, d, data, m;                                       \
        struct internal_state *state = strm->state;                      \
                                                                         \
        flush_end = state->rsip;                                         \
        if (state->pp) {                                                 \
            if (state->flush_start == state->rsi_buffer                  \
                && state->rsip > state->rsi_buffer) {                    \
                state->last_out = *state->rsi_buffer;                    \
                                                                         \
                if (strm->flags & AEC_DATA_SIGNED) {                     \
                    m = UINT32_C(1) << (strm->bits_per_sample - 1);      \
                    /* Reference samples have to be sign extended */     \
                    state->last_out = (state->last_out ^ m) - m;         \
                }                                                        \
                put_##KIND(strm, (uint32_t)state->last_out);             \
                state->flush_start++;                                    \
            }                                                            \
                                                                         \
            data = state->last_out;                                      \
            xmax = state->xmax;                                          \
                                                                         \
            if (state->xmin == 0) {                                      \
                uint32_t med;                                            \
                med = state->xmax / 2 + 1;                               \
                                                                         \
                for (bp = state->flush_start; bp < flush_end; bp++) {    \
                    uint32_t mask;                                       \
                    d = *bp;                                             \
                    half_d = (d >> 1) + (d & 1);                         \
                    /*in this case: data >= med == data & med */         \
                    mask = (data & med)?xmax:0;                          \
                                                                         \
                    /*in this case: xmax - data == xmax ^ data */        \
                    if (half_d <= (mask ^ data)) {                       \
                        data += (d >> 1)^(~((d & 1) - 1));               \
                    } else {                                             \
                        data = mask ^ d;                                 \
                    }                                                    \
                    put_##KIND(strm, data);                              \
                }                                                        \
                state->last_out = data;                                  \
            } else {                                                     \
                for (bp = state->flush_start; bp < flush_end; bp++) {    \
                    d = *bp;                                             \
                    half_d = (d >> 1) + (d & 1);                         \
                                                                         \
                    if ((int32_t)data < 0) {                             \
                        if (half_d <= xmax + data + 1) {                 \
                            data += (d >> 1)^(~((d & 1) - 1));           \
                        } else {                                         \
                            data = d - xmax - 1;                         \
                        }                                                \
                    } else {                                             \
                        if (half_d <= xmax - data) {                     \
                            data += (d >> 1)^(~((d & 1) - 1));           \
                        } else {                                         \
                            data = xmax - d;                             \
                        }                                                \
                    }                                                    \
                    put_##KIND(strm, data);                              \
                }                                                        \
                state->last_out = data;                                  \
            }                                                            \
        } else {                                                         \
            for (bp = state->flush_start; bp < flush_end; bp++)          \
                put_##KIND(strm, *bp);                                   \
        }                                                                \
        state->flush_start = state->rsip;                                \
    }
     */

    // Generic flush method that handles all byte orders
    pub fn flush_kind(&mut self, byte_order: ByteOrder) {
        let flush_end = self.rsip;

        if self.pp {
            // Handle preprocessed data
            if self.flush_start == 0 && self.rsip > 0 {
                let mut last_out = self.rsi_buffer[0];

                // Handle signed data
                if self.flags.intersects(Flags::AEC_DATA_SIGNED) {
                    let m = 1u32 << (self.bits_per_sample - 1);
                    last_out = (last_out ^ m).wrapping_sub(m);
                }

                byte_order.put_bytes(last_out, &mut self.next_out);
                self.flush_start += 1;
            }

            let mut data: u32 = self.last_out as u32;
            let xmax = self.xmax;

            if self.xmin == 0 {
                // Handle unsigned case
                let med = (self.xmax as u32) / 2 + 1;

                for bp in self.rsi_buffer[self.flush_start..flush_end].iter() {
                    let d = *bp;
                    let half_d = (d >> 1) + (d & 1);
                    let mask = if (data & med) != 0 { xmax } else { 0 };

                    data = if half_d <= (mask ^ data) {
                        data.wrapping_add((d >> 1) ^ (!((d & 1).wrapping_sub(1))))
                    } else {
                        mask ^ d
                    };

                    byte_order.put_bytes(data, &mut self.next_out);
                }
            } else {
                // Handle signed case
                for bp in self.rsi_buffer[self.flush_start..flush_end].iter() {
                    let d = *bp;
                    let half_d = (d >> 1) + (d & 1);

                    data = if (data as i32) < 0 {
                        if half_d <= xmax.wrapping_add(data).wrapping_add(1) {
                            data.wrapping_add((d >> 1) ^ (!((d & 1).wrapping_sub(1))))
                        } else {
                            d.wrapping_sub(xmax).wrapping_sub(1)
                        }
                    } else {
                        if half_d <= xmax.wrapping_sub(data) {
                            data.wrapping_add((d >> 1) ^ (!((d & 1).wrapping_sub(1))))
                        } else {
                            xmax.wrapping_sub(d)
                        }
                    };

                    byte_order.put_bytes(data, &mut self.next_out);
                }
            }
            self.last_out = data as i32;
        } else {
            // Handle non-preprocessed data
            for &bp in self.rsi_buffer[self.flush_start..flush_end].iter() {
                byte_order.put_bytes(bp, &mut self.next_out);
            }
        }

        self.flush_start = self.rsip;
    }

    pub fn flush(&mut self) {
        // Choose the appropriate byte order based on bits_per_sample and flags
        let byte_order = match self.bytes_per_sample {
            4 => {
                if !self.flags.intersects(Flags::AEC_DATA_MSB) {
                    ByteOrder::LSB32
                } else {
                    ByteOrder::MSB32
                }
            }
            3 => {
                if !self.flags.intersects(Flags::AEC_DATA_MSB) {
                    ByteOrder::LSB24
                } else {
                    ByteOrder::MSB24
                }
            }
            2 => {
                if !self.flags.intersects(Flags::AEC_DATA_MSB) {
                    ByteOrder::LSB16
                } else {
                    ByteOrder::MSB16
                }
            }
            1 => ByteOrder::Byte8,
            _ => return, // Invalid bytes_per_sample
        };

        self.flush_kind(byte_order);
    }

    fn buffer_space(&self) -> bool {
        self.avail_in >= (self.in_blklen as usize) && self.avail_out >= (self.out_blklen as usize)
    }

    /*
        static inline uint32_t bits_ask(struct aec_stream *strm, int n)
    {
        while (strm->state->bitp < n) {
            if (strm->avail_in == 0)
                return 0;
            strm->avail_in--;
            strm->state->acc <<= 8;
            strm->state->acc |= *strm->next_in++;
            strm->state->bitp += 8;
        }
        return 1;
    }

    static inline uint32_t bits_get(struct aec_stream *strm, int n)
    {
        return (strm->state->acc >> (strm->state->bitp - n))
            & (UINT64_MAX >> (64 - n));
    }

    static inline void bits_drop(struct aec_stream *strm, int n)
    {
        strm->state->bitp -= n;
    }

    static inline uint32_t fs_ask(struct aec_stream *strm)
    {
        if (bits_ask(strm, 1) == 0)
            return 0;
        while ((strm->state->acc & (UINT64_C(1) << (strm->state->bitp - 1))) == 0) {
            if (strm->state->bitp == 1) {
                if (strm->avail_in == 0)
                    return 0;
                strm->avail_in--;
                strm->state->acc <<= 8;
                strm->state->acc |= *strm->next_in++;
                strm->state->bitp += 8;
            }
            strm->state->fs++;
            strm->state->bitp--;
        }
        return 1;
    }
         */

    fn bits_get(&mut self, n: u8) -> u64 {
        (self.acc >> (self.bitp - n as usize)) & (std::u64::MAX >> (64 - n))
    }

    /*
    static inline uint32_t bits_ask(struct aec_stream *strm, int n)
{
    while (strm->state->bitp < n) {
        if (strm->avail_in == 0)
            return 0;
        strm->avail_in--;
        strm->state->acc <<= 8;
        strm->state->acc |= *strm->next_in++;
        strm->state->bitp += 8;
    }
    return 1;
} */
    fn bits_ask(&mut self, n: u8) -> bool {
        while self.bitp < n as usize {
            if self.avail_in == 0 {
                return false;
            }
            self.avail_in -= 1;
            // let actual_next_in = self.next_in[0];
            let actual_next_in = self.next_in.pop_front().unwrap();
            self.acc = self.acc << 8;
            self.acc |= actual_next_in as u64;
            // self.acc = (self.acc << 8) | (self.next_in.pop_front().unwrap() as u64);
            // Get the mask from next_in
            self.bitp += 8;
        }
        true
    }

    fn bits_drop(&mut self, n: u8) {
        self.bitp -= n as usize;
    }

    fn fs_drop(&mut self) {
        self.fs = 0;
        self.bitp -= 1;
    }
    fn fs_ask(&mut self) -> bool {
        if self.bits_ask(1) == false {
            return false;
        }
        while         (self.acc & (1 << (self.bitp - 1))) == 0        {
            if self.bitp == 1 {
                if self.avail_in == 0 {
                    return false;
                }
                self.avail_in -= 1;
                self.acc = (self.acc << 8) | (self.next_in.pop_front().unwrap() as u64);
                self.bitp += 8;
            }
            self.fs += 1;
            self.bitp -= 1;
        }
        true
    }

    fn rsi_used_size(&self) -> usize {
        self.rsip
    }

    /*
        static inline void fs_drop(struct aec_stream *strm)
    {
        strm->state->fs = 0;
        strm->state->bitp--;
    }

    static inline uint32_t copysample(struct aec_stream *strm)
    {
        if (bits_ask(strm, strm->bits_per_sample) == 0
            || strm->avail_out < strm->state->bytes_per_sample)
            return 0;

        put_sample(strm, bits_get(strm, strm->bits_per_sample));
        bits_drop(strm, strm->bits_per_sample);
        return 1;
    } */
    fn copysample(&mut self) -> u32 {
        if self.avail_out < self.bytes_per_sample {
            return 0;
        }
        // let sample = self.direct_get(self.bits_per_sample as u8) as u32;
        // self.bits_drop(self.bits_per_sample as u8);
        // sample
        let sample = self.bits_get(self.bits_per_sample as u8) as u32;
        self.put_sample(sample);
        self.bits_drop(self.bits_per_sample as u8);
        return 1;
    }

    fn put_sample(&mut self, sample: u32) {
        self.rsi_buffer[self.rsip] = sample;
        self.rsip += 1;
        self.avail_out -= self.bytes_per_sample;
        self.sample_counter -= 1;
    }


    /*
    static inline uint32_t direct_get_fs(struct aec_stream *strm)
{
    /**
       Interpret a Fundamental Sequence from the input buffer.

       Essentially counts the number of 0 bits until a 1 is
       encountered.
     */

    uint32_t fs = 0;
    struct internal_state *state = strm->state;

    if (state->bitp)
        state->acc &= UINT64_MAX >> (64 - state->bitp);
    else
        state->acc = 0;

    while (state->acc == 0) {
        if (strm->avail_in < 7)
            return 0;

        state->acc = (state->acc << 56)
            | ((uint64_t)strm->next_in[0] << 48)
            | ((uint64_t)strm->next_in[1] << 40)
            | ((uint64_t)strm->next_in[2] << 32)
            | ((uint64_t)strm->next_in[3] << 24)
            | ((uint64_t)strm->next_in[4] << 16)
            | ((uint64_t)strm->next_in[5] << 8)
            | (uint64_t)strm->next_in[6];
        strm->next_in += 7;
        strm->avail_in -= 7;
        fs += state->bitp;
        state->bitp = 56;
    }

    {
#ifndef __has_builtin
#define __has_builtin(x) 0  /* Compatibility with non-clang compilers. */
#endif
#if HAVE_DECL___BUILTIN_CLZLL || __has_builtin(__builtin_clzll)
        int i = 63 - __builtin_clzll(state->acc);
#elif defined HAVE_BSR64
        unsigned long i;
        _BitScanReverse64(&i, state->acc);
#else
        int i = state->bitp - 1;
        while ((state->acc & (UINT64_C(1) << i)) == 0)
            i--;
#endif
        fs += state->bitp - i - 1;
        state->bitp = i;
    }
    return fs;
}


 */

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

        self.acc = (self.acc << 56)
            | ((self.next_in[0] as u64) << 48)
            | ((self.next_in[1] as u64) << 40)
            | ((self.next_in[2] as u64) << 32)
            | ((self.next_in[3] as u64) << 24)
            | ((self.next_in[4] as u64) << 16)
            | ((self.next_in[5] as u64) << 8)
            | (self.next_in[6] as u64);
        self.next_in.drain(..7);
        self.avail_in -= 7;
        fs += self.bitp as u32;
        self.bitp = 56;
    }
    // let i = 63 - __builtin_clzll(self.acc);
    let mut i = self.bitp - 1;
    while (self.acc & (1 << i)) == 0 {
        i -= 1;
    }
    fs += (self.bitp - i - 1) as u32;
    self.bitp = i;
    fs
 }













    /*

       if (state->bitp < n)
    {
        int b = (63 - state->bitp) >> 3;
        if (b == 6) {
            state->acc = (state->acc << 48)
                | ((uint64_t)strm->next_in[0] << 40)
                | ((uint64_t)strm->next_in[1] << 32)
                | ((uint64_t)strm->next_in[2] << 24)
                | ((uint64_t)strm->next_in[3] << 16)
                | ((uint64_t)strm->next_in[4] << 8)
                | (uint64_t)strm->next_in[5];
        } else if (b == 7) {
            state->acc = (state->acc << 56)
                | ((uint64_t)strm->next_in[0] << 48)
                | ((uint64_t)strm->next_in[1] << 40)
                | ((uint64_t)strm->next_in[2] << 32)
                | ((uint64_t)strm->next_in[3] << 24)
                | ((uint64_t)strm->next_in[4] << 16)
                | ((uint64_t)strm->next_in[5] << 8)
                | (uint64_t)strm->next_in[6];
        } else if (b == 5) {
            state->acc = (state->acc << 40)
                | ((uint64_t)strm->next_in[0] << 32)
                | ((uint64_t)strm->next_in[1] << 24)
                | ((uint64_t)strm->next_in[2] << 16)
                | ((uint64_t)strm->next_in[3] << 8)
                | (uint64_t)strm->next_in[4];
        } else if (b == 4) {
            state->acc = (state->acc << 32)
                | ((uint64_t)strm->next_in[0] << 24)
                | ((uint64_t)strm->next_in[1] << 16)
                | ((uint64_t)strm->next_in[2] << 8)
                | (uint64_t)strm->next_in[3];
        } else if (b == 3) {
            state->acc = (state->acc << 24)
                | ((uint64_t)strm->next_in[0] << 16)
                | ((uint64_t)strm->next_in[1] << 8)
                | (uint64_t)strm->next_in[2];
        } else if (b == 2) {
            state->acc = (state->acc << 16)
                | ((uint64_t)strm->next_in[0] << 8)
                | (uint64_t)strm->next_in[1];
        } else if (b == 1) {
            state->acc = (state->acc << 8)
                | (uint64_t)strm->next_in[0];
        }
        strm->next_in += b;
        strm->avail_in -= b;
        state->bitp += b << 3;
    }

    state->bitp -= n;
    return (state->acc >> state->bitp) & (UINT64_MAX >> (64 - n));
    */

    pub fn direct_get(&mut self, n: u8) -> u32 {
        // Read n bits directly from input
        if self.bitp < n as usize {
            let b = (63 - self.bitp) >> 3;
            match b {
                7 => {
                    self.acc = (self.acc << 56)
                        | ((self.next_in[0] as u64) << 48)
                        | ((self.next_in[1] as u64) << 40)
                        | ((self.next_in[2] as u64) << 32)
                        | ((self.next_in[3] as u64) << 24)
                        | ((self.next_in[4] as u64) << 16)
                        | ((self.next_in[5] as u64) << 8)
                        | (self.next_in[6] as u64);
                }
                6 => {
                    self.acc = (self.acc << 48)
                        | ((self.next_in[0] as u64) << 40)
                        | ((self.next_in[1] as u64) << 32)
                        | ((self.next_in[2] as u64) << 24)
                        | ((self.next_in[3] as u64) << 16)
                        | ((self.next_in[4] as u64) << 8)
                        | (self.next_in[5] as u64);
                }
                5 => {
                    self.acc = (self.acc << 40)
                        | ((self.next_in[0] as u64) << 32)
                        | ((self.next_in[1] as u64) << 24)
                        | ((self.next_in[2] as u64) << 16)
                        | ((self.next_in[3] as u64) << 8)
                        | (self.next_in[4] as u64);
                }
                4 => {
                    self.acc = (self.acc << 32)
                        | ((self.next_in[0] as u64) << 24)
                        | ((self.next_in[1] as u64) << 16)
                        | ((self.next_in[2] as u64) << 8)
                        | (self.next_in[3] as u64);
                }
                3 => {
                    self.acc = (self.acc << 24)
                        | ((self.next_in[0] as u64) << 16)
                        | ((self.next_in[1] as u64) << 8)
                        | (self.next_in[2] as u64);
                }
                2 => {
                    self.acc = (self.acc << 16)
                        | ((self.next_in[0] as u64) << 8)
                        | (self.next_in[1] as u64);
                }
                1 => {
                    self.acc = (self.acc << 8) | (self.next_in[0] as u64);
                }
                _ => unreachable!(),
            }
            self.next_in.drain(..b);
            self.avail_in -= b;
            self.bitp += b << 3;
        }
        self.bitp -= n as usize;
        
        ((self.acc >> self.bitp) & (std::u64::MAX >> (64 - n as u64))) as u32
    }
}

enum DecodeStatus {
    Continue,
    Exit,
    Error(String),
}

impl InternalState {
    /*

    static inline int m_id(struct aec_stream *strm)
    {
        struct internal_state *state = strm->state;
        if (strm->avail_in >= strm->state->in_blklen) {
            state->id = direct_get(strm, state->id_len);
        } else {
            if (bits_ask(strm, state->id_len) == 0) {
                state->mode = m_id;
                return M_EXIT;
            }
            state->id = bits_get(strm, state->id_len);
            bits_drop(strm, state->id_len);
        }
        state->mode = state->id_table[state->id];
        return(state->mode(strm));
    }
     */
    fn run_id(&mut self) -> DecodeStatus {
        if self.avail_in >= self.in_blklen.try_into().unwrap() {
            self.id = self.direct_get(self.id_len as u8) as usize;
        } else {
            if !self.bits_ask(self.id_len as u8) {
                self.mode = Mode::Id;
                return DecodeStatus::Exit;
            }
            self.id = self.bits_get(self.id_len as u8) as usize;
            self.bits_drop(self.id_len as u8);
        }

        self.mode = self.id_table[self.id].clone();
        DecodeStatus::Continue
    }

    /*
        static int m_low_entropy_ref(struct aec_stream *strm)
    {
        struct internal_state *state = strm->state;

        if (state->ref && copysample(strm) == 0)
            return M_EXIT;

        if (state->id == 1)
            state->mode = m_se;
        else
            state->mode = m_zero_block;
        return M_CONTINUE;
    }

    static int m_low_entropy(struct aec_stream *strm)
    {
        struct internal_state *state = strm->state;

        if (bits_ask(strm, 1) == 0)
            return M_EXIT;
        state->id = bits_get(strm, 1);
        bits_drop(strm, 1);
        state->mode = m_low_entropy_ref;
        return M_CONTINUE;
    }

    */

    fn run_low_entropy_ref(&mut self) -> DecodeStatus {
        if self.reff != 0 && self.copysample() == 0 {
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
        self.id = self.bits_get(1) as usize;
        self.bits_drop(1);
        self.mode = Mode::LowEntropyRef;
        DecodeStatus::Continue
    }
    /**
         * static int m_zero_output(struct aec_stream *strm)
    {
        struct internal_state *state = strm->state;

        do {
            if (strm->avail_out < state->bytes_per_sample)
                return M_EXIT;
            put_sample(strm, 0);
        } while(--state->sample_counter);

        state->mode = m_next_cds;
        return M_CONTINUE;
    }

    static int m_zero_block(struct aec_stream *strm)
    {
        struct internal_state *state = strm->state;
        uint32_t zero_blocks;
        uint32_t zero_samples;
        uint32_t zero_bytes;

        if (fs_ask(strm) == 0)
            return M_EXIT;

        zero_blocks = state->fs + 1;
        fs_drop(strm);

        if (zero_blocks == ROS) {
            int b = (int)RSI_USED_SIZE(state) / strm->block_size;
            zero_blocks = MIN((int)(strm->rsi - b), 64 - (b % 64));
        } else if (zero_blocks > ROS) {
            zero_blocks--;
        }

        zero_samples = zero_blocks * strm->block_size - state->ref;
        if (state->rsi_size - RSI_USED_SIZE(state) < zero_samples)
            return M_ERROR;

        zero_bytes = zero_samples * state->bytes_per_sample;
        if (strm->avail_out >= zero_bytes) {
            memset(state->rsip, 0, zero_samples * sizeof(uint32_t));
            state->rsip += zero_samples;
            strm->avail_out -= zero_bytes;
            state->mode = m_next_cds;
        } else {
            state->sample_counter = zero_samples;
            state->mode = m_zero_output;
        }
        return M_CONTINUE;
    }
         */

    fn run_zero_output(&mut self) -> DecodeStatus {
        while self.sample_counter > 0 {
            if self.avail_out < self.bytes_per_sample {
                return DecodeStatus::Exit;
            }
            self.put_sample(0);
        }
        self.mode = Mode::NextCds;
        DecodeStatus::Continue
    }

    fn run_zero_block(&mut self) -> DecodeStatus {
        if !self.fs_ask() {
            return DecodeStatus::Exit;
        }
        let mut zero_blocks: usize = (self.fs + 1).try_into().unwrap();
        self.fs_drop();

        if zero_blocks == ROS {
            let b = self.rsi_used_size() / self.block_size;
            zero_blocks = std::cmp::min(self.rsi - b, 64 - (b % 64));
        } else if zero_blocks > ROS {
            zero_blocks -= 1;
        }

        let zero_samples = zero_blocks * self.block_size - self.reff;
        if (self.rsi_size - self.rsi_used_size()) < zero_samples {
            return DecodeStatus::Error(format!("Not enough space to write zero samples: size {} used {} needed {}", self.rsi_size, self.rsi_used_size(), zero_samples));
        }
        let zero_bytes = zero_samples * self.bytes_per_sample;
        if self.avail_out >= zero_bytes {
            for _ in 0..zero_samples {
                self.rsi_buffer[self.rsip] = 0;
                self.rsip += 1;
            }
            self.avail_out -= zero_bytes;
            self.mode = Mode::NextCds;
        } else {
            self.sample_counter = zero_samples as u32;
            self.mode = Mode::ZeroOutput;
        }
        DecodeStatus::Continue
    }
    /**
    static int m_se(struct aec_stream *strm)
    {
        struct internal_state *state = strm->state;

        if (BUFFERSPACE(strm)) {
            uint32_t i = state->ref;

            while (i < strm->block_size) {
                uint32_t m = direct_get_fs(strm);
                int32_t d1;

                if (m > SE_TABLE_SIZE)
                    return M_ERROR;

                d1 = m - state->se_table[2 * m + 1];

                if ((i & 1) == 0) {
                    put_sample(strm, state->se_table[2 * m] - d1);
                    i++;
                }
                put_sample(strm, d1);
                i++;
            }
            state->mode = m_next_cds;
        } else {
            state->mode = m_se_decode;
            state->sample_counter = state->ref;
        }
        return M_CONTINUE;
    }

         */

    fn run_se(&mut self) -> DecodeStatus {
        if self.buffer_space() {
            // We have enough output buffer space
            let mut i: u32 = self.reff as u32;

            while i < self.block_size as u32 {
                // Get the next value from input stream
                let m = self.direct_get_fs() as u32;

                // Validate m is within bounds
                if m > SE_TABLE_SIZE {
                    return DecodeStatus::Error(format!("SE table index out of bounds (se) {}", m));
                }

                let d1 = m as i32 - self.se_table[2 * m as usize + 1];

                // Handle even-numbered samples
                if (i & 1) == 0 {
                    self.put_sample((self.se_table[2 * m as usize] - d1) as u32);
                    i += 1;
                }

                // Handle all samples
                self.put_sample(d1 as u32);
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
    
    /*
             * static int m_se_decode(struct aec_stream *strm)
    {
        struct internal_state *state = strm->state;

        while(state->sample_counter < strm->block_size) {
            int32_t m;
            int32_t d1;
            if (fs_ask(strm) == 0)
                return M_EXIT;
            m = state->fs;
            if (m > SE_TABLE_SIZE)
                return M_ERROR;
            d1 = m - state->se_table[2 * m + 1];

            if ((state->sample_counter & 1) == 0) {
                if (strm->avail_out < state->bytes_per_sample)
                    return M_EXIT;
                put_sample(strm, state->se_table[2 * m] - d1);
                state->sample_counter++;
            }

            if (strm->avail_out < state->bytes_per_sample)
                return M_EXIT;
            put_sample(strm, d1);
            state->sample_counter++;
            fs_drop(strm);
        }

        state->mode = m_next_cds;
        return M_CONTINUE;
    }
 */
    // You might also want to implement a separate method for incremental processing:
    fn run_se_decode(&mut self) -> DecodeStatus {
        while self.sample_counter < self.block_size as u32 {
            // Get next value from input stream
            let m = self.direct_get(8) as u32;

            // Validate m is within bounds
            if m > SE_TABLE_SIZE {
                return DecodeStatus::Error(format!("SE table index out of bounds (se_decode) {}", m));
            }

            let d1 = m as i32 - self.se_table[2 * m as usize + 1];

            // Handle even-numbered samples
            if (self.sample_counter & 1) == 0 {
                if self.avail_out < self.bytes_per_sample {
                    return DecodeStatus::Exit;
                }
                self.put_sample((self.se_table[2 * m as usize] - d1) as u32);
                self.sample_counter += 1;
            }

            // Handle all samples
            if self.avail_out < self.bytes_per_sample {
                return DecodeStatus::Exit;
            }
            self.put_sample(d1 as u32);
            self.sample_counter += 1;
            self.fs_drop();
        }

        self.mode = Mode::NextCds;
        DecodeStatus::Continue
    }

    /**
         * static int m_uncomp_copy(struct aec_stream *strm)
    {
        struct internal_state *state = strm->state;

        do {
            if (copysample(strm) == 0)
                return M_EXIT;
        } while(--state->sample_counter);

        state->mode = m_next_cds;
        return M_CONTINUE;
    }

    static int m_uncomp(struct aec_stream *strm)
    {
        struct internal_state *state = strm->state;

        if (BUFFERSPACE(strm)) {
            for (size_t i = 0; i < strm->block_size; i++)
                *state->rsip++ = direct_get(strm, strm->bits_per_sample);
            strm->avail_out -= state->out_blklen;
            state->mode = m_next_cds;
        } else {
            state->sample_counter = strm->block_size;
            state->mode = m_uncomp_copy;
        }
        return M_CONTINUE;
    }
         */
    fn run_uncomp(&mut self) -> DecodeStatus {
        if self.buffer_space() {
            // We have enough output buffer space to process the entire block at once
            for _ in 0..self.block_size {
                let sample = self.direct_get(self.bits_per_sample as u8);
                self.rsi_buffer[self.rsip] = sample as u32;
                self.rsip += 1;
            }
            self.avail_out -= self.out_blklen as usize;
            self.mode = Mode::NextCds;
        } else {
            // Not enough output space, switch to incremental processing
            self.sample_counter = self.block_size as u32;
            self.mode = Mode::UncompCopy;
        }
        DecodeStatus::Continue
    }

    // Add this method for incremental processing
    fn run_uncomp_copy(&mut self) -> DecodeStatus {
        while self.sample_counter > 0 {
            if self.avail_out < self.bytes_per_sample {
                return DecodeStatus::Exit;
            }

            let sample = self.direct_get(self.bits_per_sample as u8);
            self.put_sample(sample as u32);
            self.avail_out -= self.bytes_per_sample;
            self.sample_counter -= 1;
        }

        self.mode = Mode::NextCds;
        DecodeStatus::Continue
    }

    /**
         *
         *
         static int m_split(struct aec_stream *strm)
    {
        struct internal_state *state = strm->state;

        if (BUFFERSPACE(strm)) {
            int k = state->id - 1;
            size_t binary_part = (k * state->encoded_block_size) / 8 + 9;

            if (state->ref)
                *state->rsip++ = direct_get(strm, strm->bits_per_sample);

            for (size_t i = 0; i < state->encoded_block_size; i++)
                state->rsip[i] = direct_get_fs(strm) << k;

            if (k) {
                if (strm->avail_in < binary_part)
                    return M_ERROR;

                for (size_t i = 0; i < state->encoded_block_size; i++)
                    *state->rsip++ += direct_get(strm, k);
            } else {
                state->rsip += state->encoded_block_size;
            }

            strm->avail_out -= state->out_blklen;
            state->mode = m_next_cds;
        } else {
            if (state->ref && (copysample(strm) == 0))
                return M_EXIT;
            state->sample_counter = 0;
            state->mode = m_split_fs;
        }
        return M_CONTINUE;
    }
         */
    fn run_split(&mut self) -> DecodeStatus {
        if self.buffer_space() {
            // Process entire block at once when we have enough buffer space
            let k = self.id.saturating_sub(1);
            let binary_part = (k * self.encoded_block_size as usize) / 8 + 9;

            // Handle reference sample if needed
            if self.reff != 0 {
                let sample = self.direct_get(self.bits_per_sample as u8);
                self.rsi_buffer[self.rsip] = sample as u32;
                self.rsip += 1;
            }

            // First pass: get fundamental sequence values
            // let mut base_values = Vec::with_capacity(self.encoded_block_size as usize);
            // for _ in 0..self.encoded_block_size {
            //     let fs = self.direct_get(8);
            //     base_values.push(fs << k);
            // }
            for i in 0..self.encoded_block_size {
                let fs = self.direct_get(8);
                self.rsi_buffer[self.rsip + i as usize] = fs << k;
            }

            // Second pass: add remainder bits if k > 0
            if k > 0 {
                if self.avail_in < binary_part {
                    return DecodeStatus::Error("Insufficient input for binary part".into());
                }

                for _ in 0..self.encoded_block_size {
                    let remainder = self.direct_get(k as u8);
                    self.rsi_buffer[self.rsip] = remainder;
                    self.rsip += 1;
                }
            } else {
                // No remainder bits, just copy base values
                self.rsip += self.encoded_block_size as usize;
            }

            self.avail_out -= self.out_blklen as usize;
            self.mode = Mode::NextCds;
        } else {
            // Not enough output space, switch to incremental processing
            // if self.sample_counter > 0 {
            //     let sample = self.direct_get(self.bits_per_sample as u8);
            //     if self.avail_out < self.bytes_per_sample {
            //         return DecodeStatus::Exit;
            //     }
            //     self.rsip.push(sample as u32);
            // }
            if (self.reff != 0) && self.copysample() == 0 {
                return DecodeStatus::Exit;
            }
            self.sample_counter = 0;
            self.mode = Mode::SplitFs;
        }

        DecodeStatus::Continue
    }

    /*

    static int m_split_fs(struct aec_stream *strm)
    {
        struct internal_state *state = strm->state;
        int k = state->id - 1;

        do {
            if (fs_ask(strm) == 0)
                return M_EXIT;
            state->rsip[state->sample_counter] = state->fs << k;
            fs_drop(strm);
        } while(++state->sample_counter < state->encoded_block_size);

        state->sample_counter = 0;
        state->mode = m_split_output;

        return M_CONTINUE;
    }
    */

    fn run_split_fs(&mut self) -> DecodeStatus {
        let k = self.id.saturating_sub(1);
        let mut looping = true;

        while looping {
            // Get fundamental sequence value
            // let fs = self.direct_get(8);
            if !self.fs_ask() {
                return DecodeStatus::Exit;
            }
            // Store base value
            self.rsi_buffer[self.rsip + self.sample_counter as usize] = self.fs << k;
            self.fs_drop();
            if self.sample_counter >= self.encoded_block_size {
                looping = false;
            }
            self.sample_counter += 1;
        }

        self.sample_counter = 0;
        self.mode = Mode::SplitOutput;
        DecodeStatus::Continue
    }

    /*
                 * static int m_split_output(struct aec_stream *strm)
    {
        struct internal_state *state = strm->state;
        int k = state->id - 1;

        do {
            if (bits_ask(strm, k) == 0 || strm->avail_out < state->bytes_per_sample)
                return M_EXIT;
            if (k)
                *state->rsip++ += bits_get(strm, k);
            else
                state->rsip++;
            strm->avail_out -= state->bytes_per_sample;
            bits_drop(strm, k);
        } while(++state->sample_counter < state->encoded_block_size);

        state->mode = m_next_cds;
        return M_CONTINUE;
    } */

    fn run_split_output(&mut self) -> DecodeStatus {
        let k = self.id.saturating_sub(1);
        let mut looping = true;
        while looping {
            // Check if we have enough output space
            if !self.bits_ask(k as u8) || self.avail_out < self.bytes_per_sample {
                return DecodeStatus::Exit;
            }


            // Get remainder bits if k > 0
            if k > 0 {
                let remainder = self.direct_get(k as u8);
                if remainder == 0 {
                    return DecodeStatus::Exit;
                }
                self.rsi_buffer[self.rsip] = self.bits_get(k as u8) as u32;
                self.rsip += 1;
            } else {
                // self.rsi_buffer.push(self.rsi_buffer[self.sample_counter]);
                self.rsip += 1;
            }
            // self.rsip += 1;
            self.avail_out -= self.bytes_per_sample;
            self.bits_drop(k as u8);
            if self.sample_counter >= self.encoded_block_size {
                looping = false;
            }
            self.sample_counter += 1;
        }

        self.mode = Mode::NextCds;
        DecodeStatus::Continue
    }

    /**
         * static int m_next_cds(struct aec_stream *strm)
    {
        struct internal_state *state = strm->state;

        if ((state->offsets != NULL) && (state->rsi_size == RSI_USED_SIZE(state)))
            vector_push_back(
                state->offsets,
                strm->total_in * 8 - (strm->avail_in * 8 + state->bitp));

        if (state->rsi_size == RSI_USED_SIZE(state)) {
            state->flush_output(strm);
            state->flush_start = state->rsi_buffer;
            state->rsip = state->rsi_buffer;
            if (state->pp) {
                state->ref = 1;
                state->encoded_block_size = strm->block_size - 1;
            }
            if (strm->flags & AEC_PAD_RSI)
                state->bitp -= state->bitp % 8;
        } else {
            state->ref = 0;
            state->encoded_block_size = strm->block_size;
        }
        return m_id(strm);
    }
         */
    fn run_next_cds(&mut self) -> DecodeStatus {
        // If we're tracking offsets and we've reached the RSI size
        if let Some(offsets) = &mut self.offsets {
            if self.rsi_buffer.len() == self.block_size {
                // Calculate and store bit offset
                let bit_offset = self.total_in * 8 - (self.avail_in * 8 + self.bitp);
                offsets.push(bit_offset);
            }
        }

        // Check if we've reached the RSI size
        if self.rsi_size == self.rsi_used_size() {
            // Flush output and reset buffers
            self.flush();
            // TODO: focus on this later
            self.flush_start = 0;
            // self.rsi_buffer.clear();
            self.rsi_buffer = vec![0; self.rsi_size];
            self.rsip = 0;

            // Handle preprocessing flag
            if self.pp {
                self.reff = 1;
                self.encoded_block_size = (self.block_size - 1) as u32;
            }

            // Handle RSI padding
            if self.flags.intersects(Flags::AEC_PAD_RSI) {
                // Assuming AEC_PAD_RSI = 0x01
                self.bitp -= self.bitp % 8;
            }
        } else {
            // Not at RSI boundary, prepare for next block
            self.reff = 0;
            self.encoded_block_size = self.block_size as u32;
        }

        // Switch back to ID mode
        self.mode = Mode::Id;
        DecodeStatus::Continue
    }
}

fn read_f64_from_bytes(bytes: &[u8]) -> Vec<f64> {
    // Ensure we're reading complete f64s (8 bytes each)
    let mut result = Vec::with_capacity(bytes.len() / 8);

    // Process chunks of 8 bytes
    for chunk in bytes.chunks(8) {
        if chunk.len() == 8 {
            // Only process complete f64s
            let value = f64::from_be_bytes(chunk.try_into().unwrap());
            result.push(value);
        }
    }

    result
}

fn read_u32_from_bytes(bytes: &[u8]) -> Vec<u32> {
    bytes.chunks(4).map(|chunk| u32::from_be_bytes(chunk.try_into().unwrap())).collect()
}
/*
static void create_se_table(int *table)
{
    int k = 0;
    for (int i = 0; i < 13; i++) {
        int ms = k;
        for (int j = 0; j <= i; j++) {
            table[2 * k] = i;
            table[2 * k + 1] = ms;
            k++;
        }
    }
}
     */

fn create_se_table() -> Vec<i32> {
    let mut table = vec![0; 2 * (SE_TABLE_SIZE as usize + 1)];
    let mut k: i32 = 0;
    for i in 0..13 {
        let ms = k;
        for _ in 0..=i {
            table[2 * k as usize] = i;
            table[2 * k as usize + 1] = ms;
            k += 1;
        }
    }
    table
}

pub fn extract_ccsds_data(
    data: Vec<u8>,
    block_len: u8,
    compression_options_mask: u8,
    num_samples: usize,
    reference_sample_interval: u16,
) -> Result<Vec<u32>, GribberishError> {
    println!("Initializing CCSDS decoder, input size: {:?}", data.len());
    // Prepare the input stream
    let state_or_error = InternalState::new(
        32,
        block_len as usize,
        reference_sample_interval as usize,
        Flags::from_bits_truncate(compression_options_mask),
        num_samples,
        data,
    );
    if let Err(e) = state_or_error {
        return Err(GribberishError::MessageError(e.to_string()));
    }
    let mut state = state_or_error.unwrap();

    // Initialize the internal state
    // let istaten = InternalState::new(32, block_len as usize, compression_options_mask as u32);
    // if let Err(e) = istaten {
    //     return Err(GribberishError::MessageError(e.to_string()));
    // }
    // strm.state = Some(istaten.unwrap());

    // let state = strm.state.as_mut().unwrap();

    // Decode the data
    loop {
        let status = state.run();

        match status {
            DecodeStatus::Continue => continue,
            DecodeStatus::Exit => break,
            DecodeStatus::Error(msg) => return Err(GribberishError::MessageError(msg)),
        }
    }

    println!("Finished decoding, mode: {:?}", state.mode);

    // Flush remaining data
    state.flush();

    // let mut decompressed_data = Vec::new();
    // // Collect decompressed data
    // for chunk in state.rsip.iter() {
    //     decompressed_data.push(*chunk as f64);
    // }
    let decompressed_data: Vec<u32> = read_u32_from_bytes(state.next_out.as_slice());
    // let decompressed_data = state.next_out.as_slice().load_be::<u32>();
    // let decompressed_data = (0..bits.len())
    //         .step_by(bits_per_val)
    //         .filter_map(|i| {
    //             let mut i_end_index = i + bits_per_val;
    //             if i_end_index >= bits.len() {
    //                 i_end_index = bits.len() - 1;
    //             }

    //             let relevent_bits = &bits[i..i_end_index];
    //             if relevent_bits.len() == 0 {
    //                 None
    //             } else {
    //                 Some(relevent_bits.load_be::<u32>())
    //             }
    //         });
    println!("Decompressed data size: {:?}", decompressed_data.len());
    Ok(decompressed_data)
}

// pub fn extract_ccsds_data(data: &[u8], block_len: u8, compression_options_mask: u8) -> Result<Vec<f64>, GribberishError> {

// }

// use ccsds::{ASM, FrameRSDecoder, SCID, FrameDecoder, Synchronizer, decode_framed_packets};
// use itertools::Itertools;

// use crate::error::GribberishError;

// // pub fn extract_ccsds_data(
// //     data0: Vec<u8>,
// //     block_len: u8,
// //     compression_options_mask: u8,
// // ) -> Result<Vec<f64>, GribberishError> {
// //     let izone_len = 0;
// //     let trailer_len = 0;
// //     let data = data0.as_slice();
// //     let blocks: Vec<_> = Synchronizer::new(
// //         data, &ASM.to_vec(),
// //         block_len as usize)
// //     .into_iter()
// //     .filter_map(Result::ok)
// //     .collect();

// //     // 2. Decode blocks into Frames
// //     let frames = FrameRSDecoder::builder()
// //         .interleave(1)
// //         .build()
// //         .decode(blocks.into_iter())
// //         .filter_map(Result::ok)
// //         ;

// //     let decoded_data = decode_framed_packets(
// //         frames, izone_len, trailer_len);

// //     // Map to Vec<u8>
// //     let output_data1: Vec<Vec<u8>> = decoded_data.map(|p| p.packet.data).collect_vec();

// //     // Convert each Vec<u8> to Vec<f64> and flatten the results
// //     let output_data2: Vec<f64> = output_data1.iter()
// //         .flat_map(|bytes| read_f64_from_bytes(bytes))
// //         .collect();

// //     Ok(output_data2)
// // }

// pub fn extract_ccsds_data(
//     data0: Vec<u8>,
//     block_len: u8,
//     compression_options_mask: u8,
// ) -> Result<Vec<f64>, GribberishError> {
//     let izone_len = 0;
//     let trailer_len = 0;
//     let data = data0.as_slice();
//     let blocks: Vec<_> = Synchronizer::new(
//         data, &ASM.to_vec(),
//         (block_len as usize) - &ASM.len())
//     .into_iter()
//     .filter_map(Result::ok)
//     .collect();

//     println!("BLOCKS!!-------: {:?}", blocks.len());

//     // Print the compression options mask, each bit is a flag
//     println!("compression options mask 1: {:b}, 2: {:b}, 3: {:b}, 4: {:b}, 5: {:b}, 6: {:b}, 7: {:b}, 8: {:b}", compression_options_mask & 0b00000001, compression_options_mask & 0b00000010, compression_options_mask & 0b00000100, compression_options_mask & 0b00001000, compression_options_mask & 0b00010000, compression_options_mask & 0b00100000, compression_options_mask & 0b01000000, compression_options_mask & 0b10000000);

//     // 2. Decode blocks into Frames
//     let frames = FrameDecoder::builder()
//         .build()
//         .decode(blocks.into_iter())
//         // .filter_map(Result::ok)
//         ;

//     let decoded_data = decode_framed_packets(
//         frames, izone_len, trailer_len);

//     // Map to Vec<u8>
//     let output_data1: Vec<Vec<u8>> = decoded_data.map(|p| p.packet.data).collect_vec();

//     // Convert each Vec<u8> to Vec<f64> and flatten the results
//     let output_data2: Vec<f64> = output_data1.iter()
//         .flat_map(|bytes| read_f64_from_bytes(bytes))
//         .collect();

//     Ok(output_data2)
// }

// fn read_f64_from_bytes(bytes: &[u8]) -> Vec<f64> {
//     // Ensure we're reading complete f64s (8 bytes each)
//     let mut result = Vec::with_capacity(bytes.len() / 8);

//     // Process chunks of 8 bytes
//     for chunk in bytes.chunks(8) {
//         if chunk.len() == 8 {  // Only process complete f64s
//             let value = f64::from_be_bytes(chunk.try_into().unwrap());
//             result.push(value);
//         }
//     }

//     result
// }
