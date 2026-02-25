struct Rect {
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
}

impl Rect {
    fn new(left: f32, top: f32, right: f32, bottom: f32) -> Rect
    {
        Rect { left, top, right, bottom }
    }
}