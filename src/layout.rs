pub struct Rect {
    x0: usize,
    x1: usize,
    y0: usize,
    y1: usize,
}

pub struct OverlayLayout {
    outer: Rect,
    inner: Rect,
}
