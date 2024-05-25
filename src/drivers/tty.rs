pub trait TTY {
    /// get the width
    fn width(&self) -> usize;
    /// get the height
    fn height(&self) -> usize;

    /// get the cursor position: (row, column)
    fn get_pos(&self) -> (usize, usize);
    /// set the cursor position
    fn set_pos(&mut self, row: usize, col: usize);

    /// clear the entire screen
    fn clear(&mut self);

    fn putchar(&mut self, c: u8);
}
