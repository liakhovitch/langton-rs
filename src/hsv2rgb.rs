pub fn hsv2rgb(h: u16, s: u8, v: u8) -> (u8, u8, u8){
    let sextant: u8 = (h >> 8) as u8;
    let (mut r, mut g, mut b): (u8, u8, u8) = (0, 0, 0);
    let (mut x, mut y, mut z): (&mut u8, &mut u8, &mut u8) = (&mut r, &mut g, &mut b);
    if s == 0 {
        return (v, v, v)
    }

    if sextant & 0b0010 != 0 {
        std::mem::swap(&mut x, &mut z);
    }
    if sextant & 0b0100 != 0 {
        std::mem::swap(&mut y, &mut z);
    }
    if sextant & 0b0110 == 0 {
        if sextant & 0b0001 == 0{
            std::mem::swap(&mut x, &mut y);
        }
    }
    else if sextant & 0b0001 != 0{
        std::mem::swap(&mut x, &mut y);
    }

    *y = v;

    let mut ww: u16 = v as u16 * (255 - s as u16);
    ww += 1;
    ww += ww >> 8;
    *z = (ww >> 8) as u8;

    let h_fraction = (h & 0xff) as u8;
    let mut d: u32;

    if sextant & 0b0001 == 0 {
        d = v as u32 * ((255 << 8) - (s as u16 * (256 - h_fraction as u16))) as u32;
    } else {
        d = v as u32 * ((255 << 8) - (s as u16 * (h_fraction as u16))) as u32;
    }
    d += d >> 8;
    d += v as u32;
    *x = (d >> 16) as u8;

    (r, g, b)
}