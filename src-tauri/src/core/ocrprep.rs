use crate::core::types::Frame;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Prep {
    Raw,
    Contrast,
    Mask(u8),
    NoColon,
}

impl Prep {
    pub const ALL: [Prep; 5] = [Prep::NoColon, Prep::Raw, Prep::Contrast, Prep::Mask(175), Prep::Mask(205)];

    pub fn name(self) -> &'static str {
        match self {
            Prep::Raw => "raw",
            Prep::Contrast => "contrast",
            Prep::Mask(175) => "mask-175",
            Prep::Mask(_) => "mask-205",
            Prep::NoColon => "no-colon",
        }
    }

    pub fn digits_only(self) -> bool {
        matches!(self, Prep::NoColon)
    }
}

#[derive(Debug, Clone, Copy)]
struct Comp {
    size: usize,
    x0: usize,
    x1: usize,
    y0: usize,
    y1: usize,
}

impl Comp {
    fn cx(&self) -> f32 {
        (self.x0 + self.x1) as f32 / 2.0
    }
}

fn components(f: &Frame) -> (Vec<u32>, Vec<Comp>) {
    let (w, h) = (f.w, f.h);
    let ink: Vec<bool> = f.rgba.chunks_exact(4).map(|p| p[0] < 215).collect();
    let mut label = vec![0u32; w * h];
    let mut comps = vec![Comp { size: 0, x0: 0, x1: 0, y0: 0, y1: 0 }];
    let mut stack = Vec::new();
    for start in 0..w * h {
        if !ink[start] || label[start] != 0 {
            continue;
        }
        let id = comps.len() as u32;
        let mut c = Comp { size: 0, x0: usize::MAX, x1: 0, y0: usize::MAX, y1: 0 };
        label[start] = id;
        stack.push(start);
        while let Some(i) = stack.pop() {
            let (x, y) = (i % w, i / w);
            c.size += 1;
            c.x0 = c.x0.min(x);
            c.x1 = c.x1.max(x);
            c.y0 = c.y0.min(y);
            c.y1 = c.y1.max(y);
            let mut visit = |j: usize| {
                if ink[j] && label[j] == 0 {
                    label[j] = id;
                    stack.push(j);
                }
            };
            if x > 0 {
                visit(i - 1);
            }
            if x + 1 < w {
                visit(i + 1);
            }
            if y > 0 {
                visit(i - w);
            }
            if y + 1 < h {
                visit(i + w);
            }
        }
        comps.push(c);
    }
    (label, comps)
}

fn speck_limit(comps: &[Comp]) -> usize {
    comps.iter().map(|c| c.size).max().unwrap_or(0) * 9 / 100
}

fn erase(f: &mut Frame, label: &[u32], comps: &[Comp], limit: usize) {
    for (i, &l) in label.iter().enumerate() {
        if l != 0 && comps[l as usize].size < limit {
            f.rgba[i * 4] = 255;
            f.rgba[i * 4 + 1] = 255;
            f.rgba[i * 4 + 2] = 255;
        }
    }
}

fn drop_specks(f: &mut Frame) {
    let (label, comps) = components(f);
    let limit = speck_limit(&comps);
    erase(f, &label, &comps, limit);
}

fn crop_padded(f: &Frame, x0: usize, x1: usize, y0: usize, y1: usize, pad: usize) -> Frame {
    let (cw, ch) = (x1.saturating_sub(x0) + 1, y1.saturating_sub(y0) + 1);
    let (nw, nh) = (cw + pad * 2, ch + pad * 2);
    let mut out = vec![255u8; nw * nh * 4];
    for y in 0..ch {
        for x in 0..cw {
            let i = ((y0 + y) * f.w + x0 + x) * 4;
            let o = ((y + pad) * nw + x + pad) * 4;
            out[o..o + 4].copy_from_slice(&f.rgba[i..i + 4]);
        }
    }
    Frame::new(nw, nh, out)
}

pub fn clock_groups(f: &Frame, scale: usize) -> Option<Vec<Frame>> {
    let mut c = prepare(f, Prep::Contrast, scale);
    let (label, comps) = components(&c);
    let limit = speck_limit(&comps);
    let biggest = comps.iter().map(|c| c.size).max().unwrap_or(0);
    let glyphs: Vec<Comp> = comps.iter().skip(1).copied().filter(|c| c.size >= limit).collect();
    let mut dots: Vec<Comp> = comps.iter().skip(1).copied().filter(|c| c.size < limit && c.size * 200 > biggest).collect();
    if glyphs.is_empty() || dots.len() < 2 {
        return None;
    }
    let gy0 = glyphs.iter().map(|g| g.y0).min()?;
    let gy1 = glyphs.iter().map(|g| g.y1).max()?;
    dots.retain(|d| d.y0 >= gy0.saturating_sub(2) && d.y1 <= gy1 + 2);
    dots.sort_by(|a, b| a.cx().partial_cmp(&b.cx()).unwrap_or(std::cmp::Ordering::Equal));
    let mut widths: Vec<usize> = glyphs.iter().map(|g| g.x1 - g.x0 + 1).collect();
    widths.sort_unstable();
    let glyph_w = widths[widths.len() / 2] as f32;
    let mut clusters: Vec<Vec<Comp>> = Vec::new();
    for d in dots {
        match clusters.last_mut() {
            Some(cl) if d.cx() - cl.last().map(|l| l.cx()).unwrap_or(0.0) < glyph_w * 0.6 => cl.push(d),
            _ => clusters.push(vec![d]),
        }
    }
    if clusters.len() != 2 {
        return None;
    }
    let split: Vec<usize> = clusters.iter().map(|cl| (cl.iter().map(|d| d.cx()).sum::<f32>() / cl.len() as f32) as usize).collect();
    erase(&mut c, &label, &comps, limit);
    let gx0 = glyphs.iter().map(|g| g.x0).min()?;
    let gx1 = glyphs.iter().map(|g| g.x1).max()?;
    if !(gx0 < split[0] && split[0] < split[1] && split[1] < gx1) {
        return None;
    }
    let y0 = gy0.saturating_sub(2);
    let y1 = (gy1 + 2).min(c.h - 1);
    let pad = (gy1 - gy0 + 1).max(8);
    Some(vec![
        crop_padded(&c, gx0.saturating_sub(2), split[0], y0, y1, pad),
        crop_padded(&c, split[0], split[1], y0, y1, pad),
        crop_padded(&c, split[1], (gx1 + 2).min(c.w - 1), y0, y1, pad),
    ])
}

fn sample(m: &[f32], w: usize, h: usize, sx: f32, sy: f32) -> f32 {
    let sx = sx.max(0.0);
    let sy = sy.max(0.0);
    let x0 = (sx as usize).min(w - 1);
    let y0 = (sy as usize).min(h - 1);
    let x1 = (x0 + 1).min(w - 1);
    let y1 = (y0 + 1).min(h - 1);
    let fx = sx - x0 as f32;
    let fy = sy - y0 as f32;
    m[y0 * w + x0] * (1.0 - fx) * (1.0 - fy) + m[y0 * w + x1] * fx * (1.0 - fy) + m[y1 * w + x0] * (1.0 - fx) * fy + m[y1 * w + x1] * fx * fy
}

fn render(m: &[f32], w: usize, h: usize, k: usize, pad: usize, binary: bool) -> Frame {
    let (nw, nh) = (w * k + pad * 2, h * k + pad * 2);
    let mut out = vec![255u8; nw * nh * 4];
    for y in 0..h * k {
        let sy = (y as f32 + 0.5) / k as f32 - 0.5;
        for x in 0..w * k {
            let sx = (x as f32 + 0.5) / k as f32 - 0.5;
            let ink = sample(m, w, h, sx, sy);
            let v = if binary {
                if ink >= 0.45 {
                    0
                } else {
                    255
                }
            } else {
                (255.0 * (1.0 - ink)).clamp(0.0, 255.0) as u8
            };
            let o = ((y + pad) * nw + x + pad) * 4;
            out[o] = v;
            out[o + 1] = v;
            out[o + 2] = v;
        }
    }
    Frame::new(nw, nh, out)
}

fn whiteness(f: &Frame) -> Vec<f32> {
    f.rgba.chunks_exact(4).map(|p| p[0].min(p[1]).min(p[2]) as f32).collect()
}

pub fn prepare(f: &Frame, prep: Prep, scale: usize) -> Frame {
    if f.w == 0 || f.h == 0 {
        return f.clone();
    }
    let k = scale.clamp(2, 8);
    match prep {
        Prep::NoColon => {
            let mut c = prepare(f, Prep::Contrast, scale);
            drop_specks(&mut c);
            c
        }
        Prep::Raw => f.upscale(k),
        Prep::Contrast => {
            let wv = whiteness(f);
            let mut sorted = wv.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let lo = sorted[sorted.len() * 50 / 100];
            let hi = sorted[(sorted.len() * 99 / 100).min(sorted.len() - 1)].max(lo + 1.0);
            let m: Vec<f32> = wv.iter().map(|v| ((v - lo) / (hi - lo)).clamp(0.0, 1.0)).collect();
            render(&m, f.w, f.h, k + 1, 8 * k, false)
        }
        Prep::Mask(t) => {
            let wv = whiteness(f);
            let mut m: Vec<f32> = wv.iter().map(|&v| if v >= t as f32 { 1.0 } else { 0.0 }).collect();
            let src = m.clone();
            for y in 0..f.h {
                for x in 0..f.w {
                    let i = y * f.w + x;
                    if src[i] > 0.0 {
                        continue;
                    }
                    let near = (x > 0 && src[i - 1] > 0.0) || (x + 1 < f.w && src[i + 1] > 0.0) || (y > 0 && src[i - f.w] > 0.0) || (y + 1 < f.h && src[i + f.w] > 0.0);
                    if near {
                        m[i] = 0.6;
                    }
                }
            }
            render(&m, f.w, f.h, k + 3, 10 * k, true)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame_with_white_bar() -> Frame {
        let (w, h) = (20, 10);
        let mut rgba = vec![0u8; w * h * 4];
        for y in 0..h {
            for x in 0..w {
                let i = (y * w + x) * 4;
                let white = (8..12).contains(&x);
                let c = if white { [250, 250, 250] } else { [120, 160, 210] };
                rgba[i..i + 3].copy_from_slice(&c);
                rgba[i + 3] = 255;
            }
        }
        Frame::new(w, h, rgba)
    }

    #[test]
    fn mask_turns_white_text_black_on_white_with_padding() {
        let f = frame_with_white_bar();
        let out = prepare(&f, Prep::Mask(175), 3);
        assert!(out.w > f.w * 3 && out.h > f.h * 3);
        assert_eq!(&out.rgba[0..3], &[255, 255, 255]);
        let mid = ((out.h / 2) * out.w + out.w / 2) * 4;
        assert_eq!(&out.rgba[mid..mid + 3], &[0, 0, 0]);
    }

    #[test]
    fn no_colon_removes_small_specks_but_keeps_big_glyphs() {
        let (w, h) = (40, 20);
        let mut rgba = vec![0u8; w * h * 4];
        for y in 0..h {
            for x in 0..w {
                let i = (y * w + x) * 4;
                let digit = (4..12).contains(&x) && (3..17).contains(&y);
                let dot = x == 20 && (y == 6 || y == 14);
                let c = if digit || dot { [250, 250, 250] } else { [120, 160, 210] };
                rgba[i..i + 3].copy_from_slice(&c);
                rgba[i + 3] = 255;
            }
        }
        let out = prepare(&Frame::new(w, h, rgba), Prep::NoColon, 3);
        let ink = out.rgba.chunks_exact(4).filter(|p| p[0] == 0).count();
        let contrast = prepare(&Frame::new(1, 1, vec![0, 0, 0, 255]), Prep::Contrast, 3);
        assert!(contrast.w > 0);
        assert!(ink > 0);
        let col_x = |sx: usize| {
            let k = 4usize;
            let pad = 8 * 3;
            pad + sx * k + k / 2
        };
        let dot_ink = (0..out.h).filter(|&y| out.rgba[(y * out.w + col_x(20)) * 4] == 0).count();
        let digit_ink = (0..out.h).filter(|&y| out.rgba[(y * out.w + col_x(8)) * 4] == 0).count();
        assert_eq!(dot_ink, 0);
        assert!(digit_ink > 10);
    }

    #[test]
    fn contrast_keeps_background_light() {
        let f = frame_with_white_bar();
        let out = prepare(&f, Prep::Contrast, 3);
        assert!(out.rgba[0] > 200);
    }
}
