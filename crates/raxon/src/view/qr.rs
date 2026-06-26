//! QR code generation and rendering.

use crate::core::Color;

use super::container::{column, row};
use super::modifier::ViewExt;
use super::text::text;
use super::view::{boxed, BoxedView, View};

const QUIET_ZONE: usize = 4;
const MASK_PATTERN: u8 = 0;

#[derive(Clone, Copy)]
struct VersionInfo {
    version: u8,
    size: usize,
    byte_capacity: usize,
    data_codewords: usize,
    ec_codewords: usize,
    alignment: &'static [usize],
}

const VERSION_INFOS: &[VersionInfo] = &[
    VersionInfo {
        version: 1,
        size: 21,
        byte_capacity: 17,
        data_codewords: 19,
        ec_codewords: 7,
        alignment: &[],
    },
    VersionInfo {
        version: 2,
        size: 25,
        byte_capacity: 32,
        data_codewords: 34,
        ec_codewords: 10,
        alignment: &[6, 18],
    },
    VersionInfo {
        version: 3,
        size: 29,
        byte_capacity: 53,
        data_codewords: 55,
        ec_codewords: 15,
        alignment: &[6, 22],
    },
    VersionInfo {
        version: 4,
        size: 33,
        byte_capacity: 78,
        data_codewords: 80,
        ec_codewords: 20,
        alignment: &[6, 26],
    },
];

/// A generated QR module matrix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QrMatrix {
    size: usize,
    modules: Vec<bool>,
}

impl QrMatrix {
    /// Number of modules on one side of the square QR code, without quiet zone.
    pub fn size(&self) -> usize {
        self.size
    }

    /// Returns whether the module at `(x, y)` is dark.
    pub fn get(&self, x: usize, y: usize) -> bool {
        self.modules[y * self.size + x]
    }
}

/// Error returned when QR generation cannot represent the input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QrError {
    /// The byte-mode payload is larger than this minimal encoder supports.
    DataTooLong,
}

/// Encodes `data` as a byte-mode, error-correction-level-L QR matrix.
///
/// This encoder supports versions 1 through 4, which covers compact URLs and
/// short identifiers without adding a dependency.
pub fn qr_matrix(data: &str) -> Result<QrMatrix, QrError> {
    let bytes = data.as_bytes();
    let info = VERSION_INFOS
        .iter()
        .copied()
        .find(|candidate| bytes.len() <= candidate.byte_capacity)
        .ok_or(QrError::DataTooLong)?;

    let data_codewords = build_data_codewords(bytes, info)?;
    let ec_codewords = reed_solomon_remainder(&data_codewords, info.ec_codewords);
    let mut bits = Vec::with_capacity((data_codewords.len() + ec_codewords.len()) * 8);
    for byte in data_codewords.iter().chain(ec_codewords.iter()) {
        append_bits(&mut bits, *byte as u32, 8);
    }

    let mut modules = vec![None; info.size * info.size];
    draw_function_patterns(&mut modules, info);
    draw_data(&mut modules, info.size, &bits);
    draw_format_bits(&mut modules, info.size, format_bits(MASK_PATTERN));

    Ok(QrMatrix {
        size: info.size,
        modules: modules
            .into_iter()
            .map(|module| module.unwrap_or(false))
            .collect(),
    })
}

/// A composed QR code view. Build via [`qr_code`].
pub struct QrCode {
    data: String,
    module_size: f32,
    dark: Color,
    light: Color,
}

/// Renders a QR code for `data` as a grid of square views.
pub fn qr_code(data: impl Into<String>) -> QrCode {
    QrCode {
        data: data.into(),
        module_size: 6.0,
        dark: Color::BLACK,
        light: Color::WHITE,
    }
}

impl QrCode {
    /// Sets the size of each QR module in logical points.
    #[must_use]
    pub fn module_size(mut self, size: f32) -> Self {
        self.module_size = size.max(1.0);
        self
    }

    /// Sets the dark and light colors used to render modules.
    #[must_use]
    pub fn colors(mut self, dark: Color, light: Color) -> Self {
        self.dark = dark;
        self.light = light;
        self
    }
}

impl View for QrCode {
    fn build(self, tree: &mut crate::dom::Tree) -> crate::dom::WidgetId {
        let matrix = match qr_matrix(&self.data) {
            Ok(matrix) => matrix,
            Err(QrError::DataTooLong) => {
                return text("QR data too long").color(self.dark).build(tree);
            }
        };

        let total = matrix.size() + QUIET_ZONE * 2;
        let mut rows: Vec<BoxedView> = Vec::with_capacity(total);
        for y in 0..total {
            let mut cells: Vec<BoxedView> = Vec::with_capacity(total);
            for x in 0..total {
                let dark = x >= QUIET_ZONE
                    && y >= QUIET_ZONE
                    && x < matrix.size() + QUIET_ZONE
                    && y < matrix.size() + QUIET_ZONE
                    && matrix.get(x - QUIET_ZONE, y - QUIET_ZONE);
                cells.push(boxed(
                    column(())
                        .size(self.module_size, self.module_size)
                        .background(if dark { self.dark } else { self.light }),
                ));
            }
            rows.push(boxed(row(cells)));
        }

        column(rows).background(self.light).grow_by(0.0).build(tree)
    }
}

fn build_data_codewords(data: &[u8], info: VersionInfo) -> Result<Vec<u8>, QrError> {
    if data.len() > info.byte_capacity {
        return Err(QrError::DataTooLong);
    }

    let capacity_bits = info.data_codewords * 8;
    let mut bits = Vec::new();
    append_bits(&mut bits, 0b0100, 4); // byte mode
    append_bits(&mut bits, data.len() as u32, 8);
    for byte in data {
        append_bits(&mut bits, *byte as u32, 8);
    }

    let terminator = (capacity_bits - bits.len()).min(4);
    append_bits(&mut bits, 0, terminator);
    while bits.len() % 8 != 0 {
        bits.push(false);
    }

    let mut codewords: Vec<u8> = bits
        .chunks(8)
        .map(|chunk| {
            chunk
                .iter()
                .fold(0u8, |acc, bit| (acc << 1) | u8::from(*bit))
        })
        .collect();
    let mut pad = true;
    while codewords.len() < info.data_codewords {
        codewords.push(if pad { 0xEC } else { 0x11 });
        pad = !pad;
    }
    Ok(codewords)
}

fn append_bits(bits: &mut Vec<bool>, value: u32, len: usize) {
    for i in (0..len).rev() {
        bits.push(((value >> i) & 1) != 0);
    }
}

fn reed_solomon_remainder(data: &[u8], degree: usize) -> Vec<u8> {
    let generator = reed_solomon_generator(degree);
    let mut message = Vec::with_capacity(data.len() + degree);
    message.extend_from_slice(data);
    message.resize(data.len() + degree, 0);

    for i in 0..data.len() {
        let factor = message[i];
        if factor == 0 {
            continue;
        }
        for (j, coefficient) in generator.iter().enumerate() {
            message[i + j] ^= gf_mul(*coefficient, factor);
        }
    }

    message[data.len()..].to_vec()
}

fn reed_solomon_generator(degree: usize) -> Vec<u8> {
    let mut generator = vec![1u8];
    for i in 0..degree {
        let root = gf_pow(2, i);
        let mut next = vec![0u8; generator.len() + 1];
        for (j, coefficient) in generator.iter().copied().enumerate() {
            next[j] ^= coefficient;
            next[j + 1] ^= gf_mul(coefficient, root);
        }
        generator = next;
    }
    generator
}

fn gf_pow(value: u8, power: usize) -> u8 {
    let mut result = 1u8;
    for _ in 0..power {
        result = gf_mul(result, value);
    }
    result
}

fn gf_mul(x: u8, mut y: u8) -> u8 {
    let mut z = 0u16;
    let mut a = x as u16;
    while y != 0 {
        if y & 1 != 0 {
            z ^= a;
        }
        y >>= 1;
        a <<= 1;
        if a & 0x100 != 0 {
            a ^= 0x11D;
        }
    }
    (z & 0xFF) as u8
}

fn draw_function_patterns(modules: &mut [Option<bool>], info: VersionInfo) {
    let size = info.size;
    draw_finder(modules, size, 0, 0);
    draw_finder(modules, size, size - 7, 0);
    draw_finder(modules, size, 0, size - 7);

    for i in 8..(size - 8) {
        set_function(modules, size, i, 6, i % 2 == 0);
        set_function(modules, size, 6, i, i % 2 == 0);
    }

    for &y in info.alignment {
        for &x in info.alignment {
            if modules[y * size + x].is_none() {
                draw_alignment(modules, size, x, y);
            }
        }
    }

    set_function(modules, size, 8, 4 * info.version as usize + 9, true);
    reserve_format_bits(modules, size);
}

fn draw_finder(modules: &mut [Option<bool>], size: usize, left: usize, top: usize) {
    for dy in -1isize..=7 {
        for dx in -1isize..=7 {
            let x = left as isize + dx;
            let y = top as isize + dy;
            if x < 0 || y < 0 || x >= size as isize || y >= size as isize {
                continue;
            }
            let in_pattern = (0..=6).contains(&dx) && (0..=6).contains(&dy);
            let dark = in_pattern
                && (dx == 0
                    || dx == 6
                    || dy == 0
                    || dy == 6
                    || ((2..=4).contains(&dx) && (2..=4).contains(&dy)));
            set_function(modules, size, x as usize, y as usize, dark);
        }
    }
}

fn draw_alignment(modules: &mut [Option<bool>], size: usize, center_x: usize, center_y: usize) {
    for dy in -2isize..=2 {
        for dx in -2isize..=2 {
            let distance = dx.abs().max(dy.abs());
            let dark = distance == 2 || distance == 0;
            set_function(
                modules,
                size,
                (center_x as isize + dx) as usize,
                (center_y as isize + dy) as usize,
                dark,
            );
        }
    }
}

fn reserve_format_bits(modules: &mut [Option<bool>], size: usize) {
    let coords = format_coordinates(size);
    for (x, y) in coords {
        set_function(modules, size, x, y, false);
    }
}

fn draw_format_bits(modules: &mut [Option<bool>], size: usize, bits: u16) {
    for (i, (x, y)) in format_coordinates(size).into_iter().enumerate() {
        set_function(modules, size, x, y, ((bits >> (i % 15)) & 1) != 0);
    }
}

fn format_coordinates(size: usize) -> [(usize, usize); 30] {
    [
        (8, 0),
        (8, 1),
        (8, 2),
        (8, 3),
        (8, 4),
        (8, 5),
        (8, 7),
        (8, 8),
        (7, 8),
        (5, 8),
        (4, 8),
        (3, 8),
        (2, 8),
        (1, 8),
        (0, 8),
        (size - 1, 8),
        (size - 2, 8),
        (size - 3, 8),
        (size - 4, 8),
        (size - 5, 8),
        (size - 6, 8),
        (size - 7, 8),
        (size - 8, 8),
        (8, size - 7),
        (8, size - 6),
        (8, size - 5),
        (8, size - 4),
        (8, size - 3),
        (8, size - 2),
        (8, size - 1),
    ]
}

fn draw_data(modules: &mut [Option<bool>], size: usize, bits: &[bool]) {
    let mut bit_index = 0usize;
    let mut upward = true;
    let mut right = size - 1;

    while right > 0 {
        if right == 6 {
            right -= 1;
        }

        for i in 0..size {
            let y = if upward { size - 1 - i } else { i };
            for dx in 0..2 {
                let x = right - dx;
                let index = y * size + x;
                if modules[index].is_some() {
                    continue;
                }
                let bit = bit_index < bits.len() && bits[bit_index];
                bit_index += 1;
                modules[index] = Some(bit ^ mask_bit(MASK_PATTERN, x, y));
            }
        }

        upward = !upward;
        if right < 2 {
            break;
        }
        right -= 2;
    }
}

fn mask_bit(mask: u8, x: usize, y: usize) -> bool {
    match mask {
        0 => (x + y) % 2 == 0,
        _ => false,
    }
}

fn set_function(modules: &mut [Option<bool>], size: usize, x: usize, y: usize, dark: bool) {
    modules[y * size + x] = Some(dark);
}

fn format_bits(mask: u8) -> u16 {
    let data = (0b01u16 << 3) | mask as u16; // error correction level L
    let mut rem = data << 10;
    for i in (10..=14).rev() {
        if ((rem >> i) & 1) != 0 {
            rem ^= 0x537 << (i - 10);
        }
    }
    ((data << 10) | rem) ^ 0x5412
}

#[cfg(test)]
mod tests {
    use crate::dom::{Host, RecordingBackend, Tree, WidgetKind};

    use super::*;

    #[test]
    fn format_bits_match_level_l_mask_zero() {
        assert_eq!(format_bits(0), 0x77C4);
    }

    #[test]
    fn rtylr_url_fits_in_version_one_matrix() {
        let matrix = qr_matrix("https://rtylr.com").expect("url should encode");

        assert_eq!(matrix.size(), 21);
        assert!(matrix.get(0, 0));
        assert!(matrix.get(3, 3));
        assert!(matrix.get(20, 0));
        assert!(matrix.get(8, 13), "dark module is present");
    }

    #[test]
    fn longer_payloads_select_larger_versions() {
        let matrix = qr_matrix("https://rtylr.com/orders/receipt/12345").expect("fits v3");

        assert_eq!(matrix.size(), 29);
    }

    #[test]
    fn overlarge_payloads_are_rejected() {
        let data = "x".repeat(79);

        assert_eq!(qr_matrix(&data), Err(QrError::DataTooLong));
    }

    #[test]
    fn qr_code_view_builds_a_composed_grid() {
        let backend = RecordingBackend::new();
        let mut tree = Tree::new(Host::new(backend));

        let root = qr_code("https://rtylr.com")
            .module_size(4.0)
            .build(&mut tree);

        assert_eq!(tree.kind_of(root), Some(WidgetKind::View));
        assert_eq!(tree.children_of(root).len(), 29);
    }
}
