use crate::{Sink, Source};

use super::blit_sub;

const CHARSET_WIDTH: u32 = 128;
const CHARSET_HEIGHT: u32 = 112;
const CHARSET_FLAGS: u32 = 0; // BLIT_1BPP
const CHARSET: [u8; 1792] = [
    0xff, 0xc7, 0x93, 0x93, 0xef, 0x9d, 0x8f, 0xcf, 0xf3, 0x9f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfd,
    0xff, 0xc7, 0x93, 0x01, 0x83, 0x5b, 0x27, 0xcf, 0xe7, 0xcf, 0x93, 0xe7, 0xff, 0xff, 0xff, 0xfb,
    0xff, 0xc7, 0x93, 0x93, 0x2f, 0x37, 0x27, 0xcf, 0xcf, 0xe7, 0xc7, 0xe7, 0xff, 0xff, 0xff, 0xf7,
    0xff, 0xcf, 0xff, 0x93, 0x83, 0xef, 0x8f, 0xff, 0xcf, 0xe7, 0x01, 0x81, 0xff, 0x81, 0xff, 0xef,
    0xff, 0xcf, 0xff, 0x93, 0xe9, 0xd9, 0x25, 0xff, 0xcf, 0xe7, 0xc7, 0xe7, 0xff, 0xff, 0xff, 0xdf,
    0xff, 0xff, 0xff, 0x01, 0x03, 0xb5, 0x33, 0xff, 0xe7, 0xcf, 0x93, 0xe7, 0xcf, 0xff, 0xcf, 0xbf,
    0xff, 0xcf, 0xff, 0x93, 0xef, 0x73, 0x81, 0xff, 0xf3, 0x9f, 0xff, 0xff, 0xcf, 0xff, 0xcf, 0x7f,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x9f, 0xff, 0xff, 0xff,
    0xc7, 0xe7, 0x83, 0x81, 0xe3, 0x03, 0xc3, 0x01, 0x87, 0x83, 0xff, 0xff, 0xf3, 0xff, 0x9f, 0x83,
    0xb3, 0xc7, 0x39, 0xf3, 0xc3, 0x3f, 0x9f, 0x39, 0x3b, 0x39, 0xcf, 0xcf, 0xe7, 0xff, 0xcf, 0x01,
    0x39, 0xe7, 0xf1, 0xe7, 0x93, 0x03, 0x3f, 0xf3, 0x1b, 0x39, 0xcf, 0xcf, 0xcf, 0x01, 0xe7, 0x39,
    0x39, 0xe7, 0xc3, 0xc3, 0x33, 0xf9, 0x03, 0xe7, 0x87, 0x81, 0xff, 0xff, 0x9f, 0xff, 0xf3, 0xf3,
    0x39, 0xe7, 0x87, 0xf9, 0x01, 0xf9, 0x39, 0xcf, 0x61, 0xf9, 0xcf, 0xcf, 0xcf, 0x01, 0xe7, 0xc7,
    0x9b, 0xe7, 0x1f, 0x39, 0xf3, 0x39, 0x39, 0xcf, 0x79, 0xf3, 0xcf, 0xcf, 0xe7, 0xff, 0xcf, 0xff,
    0xc7, 0x81, 0x01, 0x83, 0xf3, 0x83, 0x83, 0xcf, 0x83, 0x87, 0xff, 0x9f, 0xf3, 0xff, 0x9f, 0xc7,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0x83, 0xc7, 0x03, 0xc3, 0x07, 0x01, 0x01, 0xc1, 0x39, 0x81, 0xf9, 0x39, 0x9f, 0x39, 0x39, 0x83,
    0x7d, 0x93, 0x39, 0x99, 0x33, 0x3f, 0x3f, 0x9f, 0x39, 0xe7, 0xf9, 0x33, 0x9f, 0x11, 0x19, 0x39,
    0x45, 0x39, 0x39, 0x3f, 0x39, 0x3f, 0x3f, 0x3f, 0x39, 0xe7, 0xf9, 0x27, 0x9f, 0x01, 0x09, 0x39,
    0x55, 0x39, 0x03, 0x3f, 0x39, 0x03, 0x03, 0x31, 0x01, 0xe7, 0xf9, 0x0f, 0x9f, 0x01, 0x01, 0x39,
    0x41, 0x01, 0x39, 0x3f, 0x39, 0x3f, 0x3f, 0x39, 0x39, 0xe7, 0xf9, 0x07, 0x9f, 0x29, 0x21, 0x39,
    0x7f, 0x39, 0x39, 0x99, 0x33, 0x3f, 0x3f, 0x99, 0x39, 0xe7, 0x39, 0x23, 0x9f, 0x39, 0x31, 0x39,
    0x83, 0x39, 0x03, 0xc3, 0x07, 0x01, 0x3f, 0xc1, 0x39, 0x81, 0x83, 0x31, 0x81, 0x39, 0x39, 0x83,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0x03, 0x83, 0x03, 0x87, 0x81, 0x39, 0x39, 0x39, 0x39, 0x99, 0x01, 0xc3, 0x7f, 0x87, 0xc7, 0xff,
    0x39, 0x39, 0x39, 0x33, 0xe7, 0x39, 0x39, 0x39, 0x11, 0x99, 0xf1, 0xcf, 0xbf, 0xe7, 0x93, 0xff,
    0x39, 0x39, 0x39, 0x3f, 0xe7, 0x39, 0x39, 0x29, 0x83, 0x99, 0xe3, 0xcf, 0xdf, 0xe7, 0xff, 0xff,
    0x39, 0x39, 0x31, 0x83, 0xe7, 0x39, 0x11, 0x01, 0xc7, 0xc3, 0xc7, 0xcf, 0xef, 0xe7, 0xff, 0xff,
    0x03, 0x21, 0x07, 0xf9, 0xe7, 0x39, 0x83, 0x01, 0x83, 0xe7, 0x8f, 0xcf, 0xf7, 0xe7, 0xff, 0xff,
    0x3f, 0x33, 0x23, 0x39, 0xe7, 0x39, 0xc7, 0x11, 0x11, 0xe7, 0x1f, 0xcf, 0xfb, 0xe7, 0xff, 0xff,
    0x3f, 0x85, 0x31, 0x83, 0xe7, 0x83, 0xef, 0x39, 0x39, 0xe7, 0x01, 0xc3, 0xfd, 0x87, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01,
    0xef, 0xff, 0x3f, 0xff, 0xf9, 0xff, 0xf1, 0xff, 0x3f, 0xe7, 0xf3, 0x3f, 0xc7, 0xff, 0xff, 0xff,
    0xf7, 0xff, 0x3f, 0xff, 0xf9, 0xff, 0xe7, 0xff, 0x3f, 0xff, 0xff, 0x3f, 0xe7, 0xff, 0xff, 0xff,
    0xff, 0x83, 0x03, 0x81, 0x81, 0x83, 0x81, 0x81, 0x03, 0xc7, 0xe3, 0x31, 0xe7, 0x03, 0x03, 0x83,
    0xff, 0xf9, 0x39, 0x3f, 0x39, 0x39, 0xe7, 0x39, 0x39, 0xe7, 0xf3, 0x03, 0xe7, 0x49, 0x39, 0x39,
    0xff, 0x81, 0x39, 0x3f, 0x39, 0x01, 0xe7, 0x39, 0x39, 0xe7, 0xf3, 0x07, 0xe7, 0x49, 0x39, 0x39,
    0xff, 0x39, 0x39, 0x3f, 0x39, 0x3f, 0xe7, 0x81, 0x39, 0xe7, 0xf3, 0x23, 0xe7, 0x49, 0x39, 0x39,
    0xff, 0x81, 0x83, 0x81, 0x81, 0x83, 0xe7, 0xf9, 0x39, 0x81, 0xf3, 0x31, 0x81, 0x49, 0x39, 0x83,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x83, 0xff, 0xff, 0x87, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xe7, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xf3, 0xe7, 0x9f, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xe7, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xe7, 0xe7, 0xcf, 0xff, 0xff,
    0x03, 0x81, 0x91, 0x83, 0x81, 0x39, 0x99, 0x49, 0x39, 0x39, 0x01, 0xe7, 0xe7, 0xcf, 0x8f, 0xff,
    0x39, 0x39, 0x8f, 0x3f, 0xe7, 0x39, 0x99, 0x49, 0x01, 0x39, 0xe3, 0xcf, 0xe7, 0xe7, 0x45, 0xff,
    0x39, 0x39, 0x9f, 0x83, 0xe7, 0x39, 0x99, 0x49, 0xc7, 0x39, 0xc7, 0xe7, 0xe7, 0xcf, 0xe3, 0xff,
    0x03, 0x81, 0x9f, 0xf9, 0xe7, 0x39, 0xc3, 0x49, 0x01, 0x81, 0x8f, 0xe7, 0xe7, 0xcf, 0xff, 0x93,
    0x3f, 0xf9, 0x9f, 0x03, 0xe7, 0x81, 0xe7, 0x81, 0x39, 0xf9, 0x01, 0xf3, 0xe7, 0x9f, 0xff, 0x93,
    0x3f, 0xf9, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x83, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0x83, 0x83, 0xff, 0xff, 0x83, 0x83, 0x83, 0x83, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0x29, 0x39, 0xff, 0xff, 0x11, 0x11, 0x11, 0x11, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0x29, 0x09, 0xff, 0xff, 0x21, 0x09, 0x39, 0x11, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0x11, 0x11, 0xff, 0xff, 0x7d, 0x7d, 0x55, 0x55, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0x29, 0x21, 0xff, 0xff, 0x21, 0x09, 0x11, 0x39, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0x29, 0x39, 0xff, 0xff, 0x11, 0x11, 0x11, 0x11, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0x83, 0x83, 0xff, 0xff, 0x83, 0x83, 0x83, 0x83, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xe7, 0xef, 0xc3, 0xff, 0x99, 0xe7, 0xc3, 0x93, 0xc3, 0x87, 0xff, 0xff, 0xff, 0xc3, 0x83,
    0xff, 0xff, 0x83, 0x99, 0xa5, 0x99, 0xe7, 0x99, 0xff, 0xbd, 0xc3, 0xc9, 0xff, 0xff, 0xbd, 0xff,
    0xff, 0xe7, 0x29, 0x9f, 0xdb, 0xc3, 0xe7, 0x87, 0xff, 0x66, 0x93, 0x93, 0x81, 0xff, 0x46, 0xff,
    0xff, 0xe7, 0x2f, 0x03, 0xdb, 0x81, 0xff, 0xdb, 0xff, 0x5e, 0xc3, 0x27, 0xf9, 0xff, 0x5a, 0xff,
    0xff, 0xc7, 0x29, 0x9f, 0xdb, 0xe7, 0xe7, 0xe1, 0xff, 0x5e, 0xff, 0x93, 0xf9, 0xff, 0x46, 0xff,
    0xff, 0xc7, 0x83, 0x9f, 0xa5, 0x81, 0xe7, 0x99, 0xff, 0x66, 0xff, 0xc9, 0xff, 0xff, 0x5a, 0xff,
    0xff, 0xc7, 0xef, 0x01, 0xff, 0xe7, 0xe7, 0xc3, 0xff, 0xbd, 0xff, 0xff, 0xff, 0xff, 0xbd, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xc3, 0xff, 0xff, 0xff, 0xff, 0xc3, 0xff,
    0xef, 0xe7, 0xc7, 0xc3, 0xf7, 0xff, 0xc1, 0xff, 0xff, 0xe7, 0xc7, 0xff, 0xbd, 0xbd, 0x1d, 0xc7,
    0xd7, 0xe7, 0xf3, 0xe7, 0xef, 0xff, 0x95, 0xff, 0xff, 0xc7, 0x93, 0x27, 0x3b, 0x3b, 0xbb, 0xff,
    0xef, 0x81, 0xe7, 0xf3, 0xff, 0x33, 0xb5, 0xff, 0xff, 0xe7, 0x93, 0x93, 0xb7, 0xb7, 0xd7, 0xc7,
    0xff, 0xe7, 0xc3, 0xc7, 0xff, 0x33, 0x95, 0xcf, 0xff, 0xc3, 0xc7, 0xc9, 0xad, 0xa9, 0x2d, 0x9f,
    0xff, 0xe7, 0xff, 0xff, 0xff, 0x33, 0xc1, 0xcf, 0xff, 0xff, 0xff, 0x93, 0xd9, 0xdd, 0xd9, 0x39,
    0xff, 0xff, 0xff, 0xff, 0xff, 0x33, 0xf5, 0xff, 0xff, 0xff, 0xff, 0x27, 0xb1, 0xbb, 0xb1, 0x01,
    0xff, 0x81, 0xff, 0xff, 0xff, 0x09, 0xf5, 0xff, 0xf7, 0xff, 0xff, 0xff, 0x7d, 0x71, 0x7d, 0x83,
    0xff, 0xff, 0xff, 0xff, 0xff, 0x3f, 0xff, 0xff, 0xcf, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xdf, 0xf7, 0xc7, 0xcb, 0x93, 0xef, 0xc1, 0xc3, 0xdf, 0xf7, 0xc7, 0x93, 0xef, 0xf7, 0xe7, 0x99,
    0xef, 0xef, 0x93, 0xa7, 0xff, 0xd7, 0x87, 0x99, 0xef, 0xef, 0x93, 0xff, 0xf7, 0xef, 0xc3, 0xff,
    0xc7, 0xc7, 0xc7, 0xc7, 0xc7, 0xc7, 0x27, 0x3f, 0x01, 0x01, 0x01, 0x01, 0x81, 0x81, 0x81, 0x81,
    0x93, 0x93, 0x93, 0x93, 0x93, 0x93, 0x21, 0x3f, 0x3f, 0x3f, 0x3f, 0x3f, 0xe7, 0xe7, 0xe7, 0xe7,
    0x39, 0x39, 0x39, 0x39, 0x39, 0x39, 0x07, 0x99, 0x03, 0x03, 0x03, 0x03, 0xe7, 0xe7, 0xe7, 0xe7,
    0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x27, 0xc3, 0x3f, 0x3f, 0x3f, 0x3f, 0xe7, 0xe7, 0xe7, 0xe7,
    0x39, 0x39, 0x39, 0x39, 0x39, 0x39, 0x21, 0xf7, 0x01, 0x01, 0x01, 0x01, 0x81, 0x81, 0x81, 0x81,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xcf, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0x87, 0xcb, 0xdf, 0xf7, 0xc7, 0xcb, 0x93, 0xff, 0x83, 0xdf, 0xf7, 0xc7, 0x93, 0xf7, 0x3f, 0xc3,
    0x93, 0xa7, 0xef, 0xef, 0x93, 0xa7, 0xff, 0xbb, 0x39, 0xef, 0xef, 0x93, 0xff, 0xef, 0x03, 0x99,
    0x99, 0x19, 0x83, 0x83, 0x83, 0x83, 0x83, 0xd7, 0x31, 0x39, 0x39, 0xff, 0x39, 0x99, 0x39, 0x99,
    0x09, 0x09, 0x39, 0x39, 0x39, 0x39, 0x39, 0xef, 0x29, 0x39, 0x39, 0x39, 0x39, 0x99, 0x39, 0x93,
    0x99, 0x01, 0x39, 0x39, 0x39, 0x39, 0x39, 0xd7, 0x19, 0x39, 0x39, 0x39, 0x39, 0xc3, 0x39, 0x99,
    0x93, 0x21, 0x39, 0x39, 0x39, 0x39, 0x39, 0xbb, 0x39, 0x39, 0x39, 0x39, 0x39, 0xe7, 0x03, 0x89,
    0x87, 0x31, 0x83, 0x83, 0x83, 0x83, 0x83, 0xff, 0x83, 0x83, 0x83, 0x83, 0x83, 0xe7, 0x3f, 0x93,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xdf, 0xf7, 0xc7, 0xcb, 0x93, 0xef, 0xff, 0xff, 0xdf, 0xf7, 0xc7, 0x93, 0xdf, 0xf7, 0xc7, 0x93,
    0xef, 0xef, 0x93, 0xa7, 0xff, 0xd7, 0xff, 0xff, 0xef, 0xef, 0x93, 0xff, 0xef, 0xef, 0x93, 0xff,
    0x83, 0x83, 0x83, 0x83, 0x83, 0x83, 0x83, 0x81, 0x83, 0x83, 0x83, 0x83, 0xff, 0xff, 0xff, 0xc7,
    0xf9, 0xf9, 0xf9, 0xf9, 0xf9, 0xf9, 0xe9, 0x3f, 0x39, 0x39, 0x39, 0x39, 0xc7, 0xc7, 0xc7, 0xe7,
    0x81, 0x81, 0x81, 0x81, 0x81, 0x81, 0x81, 0x3f, 0x01, 0x01, 0x01, 0x01, 0xe7, 0xe7, 0xe7, 0xe7,
    0x39, 0x39, 0x39, 0x39, 0x39, 0x39, 0x2f, 0x81, 0x3f, 0x3f, 0x3f, 0x3f, 0xe7, 0xe7, 0xe7, 0xe7,
    0x81, 0x81, 0x81, 0x81, 0x81, 0x81, 0x83, 0xf7, 0x83, 0x83, 0x83, 0x83, 0x81, 0x81, 0x81, 0x81,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xcf, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0x9b, 0xcb, 0xdf, 0xf7, 0xc7, 0xcb, 0x93, 0xff, 0xff, 0xdf, 0xf7, 0xc7, 0x93, 0xf7, 0x3f, 0x93,
    0x87, 0xa7, 0xef, 0xef, 0x93, 0xa7, 0xff, 0xe7, 0xff, 0xef, 0xef, 0x93, 0xff, 0xef, 0x3f, 0xff,
    0x67, 0x03, 0x83, 0x83, 0x83, 0x83, 0x83, 0xff, 0x83, 0x39, 0x39, 0xff, 0x39, 0x39, 0x03, 0x39,
    0x83, 0x39, 0x39, 0x39, 0x39, 0x39, 0x39, 0x81, 0x31, 0x39, 0x39, 0x39, 0x39, 0x39, 0x39, 0x39,
    0x39, 0x39, 0x39, 0x39, 0x39, 0x39, 0x39, 0xff, 0x29, 0x39, 0x39, 0x39, 0x39, 0x39, 0x39, 0x39,
    0x39, 0x39, 0x39, 0x39, 0x39, 0x39, 0x39, 0xe7, 0x19, 0x39, 0x39, 0x39, 0x39, 0x81, 0x03, 0x81,
    0x83, 0x39, 0x83, 0x83, 0x83, 0x83, 0x83, 0xff, 0x83, 0x81, 0x81, 0x81, 0x81, 0xf9, 0x3f, 0xf9,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x83, 0x3f, 0x83,
];

/// Draws text using the built-in font.
///
/// - The string may contain new-line (`\\n`) characters.
/// - The font is 8x8 pixels per character.
pub fn text<T: Source<u8> + Sink<u8>, B: Copy + Into<usize>>(
    fb: &mut T,
    text: &[B],
    x: i32,
    y: i32,
    draw_colors: u16,
) {
    let (mut tx, mut ty) = (x, y);

    for c in text {
        let c: usize = (*c).into();

        match c {
            0 => {
                // null-terminator
                break;
            }
            10 => {
                // line feed
                ty += 8;
                tx = x;
            }
            32..=255 => {
                // weird. this is what w4 is doing...
                let src_x = (((c - 32) & 0x0f) * 8) as u32;
                let src_y = (((c - 32) >> 4) * 8) as u32;

                blit_sub(
                    fb,
                    &Vec::from(CHARSET),
                    tx,
                    ty,
                    8,
                    8,
                    src_x,
                    src_y,
                    CHARSET_WIDTH,
                    CHARSET_FLAGS,
                    draw_colors,
                );
                tx += 8;
            }
            _ => {
                tx += 8;
            }
        }
    }
}
