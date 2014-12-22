use std::io::{File, Reader, BufferedReader};

use gapbuffer::GapBuffer;

use cursor::Direction;

pub struct Buffer {
    pub file_path: Option<Path>,
    pub lines: Vec<Line>,

    pub cursor: uint,
    pub text: GapBuffer<char>,
}

impl Buffer {
    /// Create a new buffer instance
    pub fn new() -> Buffer {
        Buffer {
            file_path: None,
            lines: Vec::new(),
            cursor: 0,
            text: GapBuffer::new(),
        }
    }

    pub fn new_from_reader<R: Reader>(reader: R) -> Buffer {
        let mut buf = Buffer::new();
        if let Ok(contents) = BufferedReader::new(reader).read_to_string() {
            buf.text.extend(contents.chars());
        }
        buf
    }

    /// Create a new buffer instance and load the given file
    pub fn new_from_file(path: Path) -> Buffer {
        if let Ok(file) = File::open(&path) {
            let mut buffer = Buffer::new_from_reader(file);
            buffer.file_path = Some(path);
            buffer
        } else {
            Buffer::new()
        }
    }

    pub fn move_cursor(&mut self, offset: int) {
        let idx = self.cursor as int + offset;
        if 0 >= idx && idx > self.text.len() as int {
            self.set_cursor(idx as uint);
        }
    }

    pub fn set_cursor(&mut self, location: uint) {
        self.cursor = location;
    }

    pub fn get_status_text(&self) -> String {
        match self.file_path {
            Some(ref path) => format!("{} {}", path.display(), self.cursor),
            None => format!("untitled {}", self.cursor)
        }
    }

    //Returns the number of newlines in the buffer before the mark.
    fn get_line(&self, mark: uint) -> Option<uint> {
        let mut linenum = 0;
        if mark < self.text.len() {
            for c in self.text[0..mark].iter() {
                if c == &'\n' { linenum += 1; }
            }
            Some(linenum)
        } else { None }
    }
    
    fn get_line_idx(&self, ln: int) -> Option<uint> {
        let mut linenum = 0;
        for (index, ch) in self.text.iter().enumerate() {
            if *ch == '\n' {
                linenum += 1;
            }
            if linenum == ln {
                return Some(index)
            }
        }
        None
    }

    fn move_line(&mut self, offset: int) {
        let current_line_num = match self.get_line(self.cursor) {
            Some(n) => n as int,
            None => return,
        };

        let len = self.text.len() - 1;
        let last_line_num = match self.get_line(len) {
            Some(n) => n as int,
            None => return,
        };

        if current_line_num < last_line_num {
            self.cursor = self.get_line_idx(current_line_num + offset).unwrap();
            self.cursor += 1;
        }
    }


    //Shift the cursor by one in any of the four directions.
    pub fn shift_cursor(&mut self, direction: Direction) {
        match direction {
            Direction::Up => { self.move_line(-1); }
            Direction::Down => { self.move_line(1); }
            Direction::Left if self.cursor > 0 => {
                if self.text[self.cursor-1] != '\n' {
                    self.cursor -= 1;
                }
            }
            Direction::Right if self.cursor < self.text.len() => {
                if self.text[self.cursor+1] != '\n' {
                    self.cursor += 1;
                }
            }
            _ => { }
        }
    }

    pub fn insert_char(&mut self, ch: char) {
        self.text.insert(self.cursor, ch);

        // FIXME: see cursor.inc_offset
        self.cursor += 1;
    }

    pub fn delete_char(&mut self, dir: Direction) {
        if dir.is_left() {
            if self.cursor == 0 { return }
            // FIXME: see cursor.dev_offset
            self.cursor -= 1;
        }
        self.text.remove(self.cursor);
    }

    /// Find out how far the cursor is from the start of the line
    pub fn get_cursor_screen_offset(&self) -> uint {
        if self.cursor == 0 { return 0 }

        let text = self.text[0..self.cursor];
        for (index, ch) in text.iter().rev().enumerate() {
            if *ch == '\n' {
                panic!("test {}", index)
            }
        }
        return 0
    }

    fn get_line_at(&self, line_num: uint) -> Option<&Line> {
        let num_lines = self.lines.len() -1;
        if line_num > num_lines { return None }
        Some(&self.lines[line_num])
    }

    fn get_line_at_mut(&mut self, line_num: uint) -> Option<&mut Line> {
        let num_lines = self.lines.len() -1;
        if line_num > num_lines { return None }
        Some(&mut self.lines[line_num])
    }

}


pub struct Line {
    pub data: String,
    pub linenum: uint,
}

impl Line {
    /// Create a new line instance
    pub fn new(data: String, line_num: uint) -> Line {
        Line{
            data: data,
            linenum: line_num,
        }
    }

    /// Get the length of the current line
    pub fn len(&self) -> uint {
        self.data.len()
    }
}


#[cfg(test)]
mod tests {

    use buffer::Buffer;
    use buffer::Line;
    use utils::data_from_str;

    fn setup_buffer() -> Buffer {
        let mut buffer = Buffer::new();
        buffer.file_path = Some(Path::new("/some/file.txt"));
        buffer.lines = vec!(
            Line::new(data_from_str("test"), 0),
            Line::new(String::new(), 1),
            Line::new(data_from_str("text file"), 2),
            Line::new(data_from_str("content"), 3),
        );
        buffer
    }

    #[test]
    fn test_get_status_text() {
        let buffer = setup_buffer();
        assert_eq!(buffer.get_status_text(), "/some/file.txt, lines: 4".to_string())
    }

    #[test]
    fn test_insert_line() {
        let mut buffer = setup_buffer();
        buffer.insert_line(0, 0);
        assert_eq!(buffer.lines.len(), 5);
    }

    #[test]
    fn test_insert_line_in_middle_of_other_line() {
        let mut buffer = setup_buffer();
        buffer.insert_line(1, 0);
        assert_eq!(buffer.lines.len(), 5);

        let ref line = buffer.lines[1];
        assert_eq!(line.data, data_from_str("est"));
    }

    #[test]
    fn test_line_numbers_are_fixed_after_adding_new_line() {
        let mut buffer = setup_buffer();
        buffer.insert_line(1, 2);
        assert_eq!(buffer.lines.len(), 5);

        // check that all linenums are sequential
        for (index, line) in buffer.lines.iter().enumerate() {
            assert_eq!(index, line.linenum);
        }
    }

    #[test]
    fn test_join_line_with_previous() {
        let mut buffer = setup_buffer();

        let offset = buffer.join_line_with_previous(0, 3);

        assert_eq!(buffer.lines.len(), 3);
        assert_eq!(buffer.lines[2].data, data_from_str("text filecontent"));
        assert_eq!(offset, 9);
    }

    #[test]
    fn join_line_with_previous_does_nothing_on_line_zero_offset_zero() {
        let mut buffer = setup_buffer();
        buffer.join_line_with_previous(0, 0);

        assert_eq!(buffer.lines.len(), 4);
        assert_eq!(buffer.lines[0].data, data_from_str("test"));
    }

    #[test]
    fn test_split_line() {
        let mut buffer = setup_buffer();
        let (old, new) = buffer.split_line(3, 3);

        assert_eq!(old, data_from_str("con"));
        assert_eq!(new, data_from_str("tent"));
    }

    #[test]
    fn joining_line_with_non_existant_next_line_does_nothing() {
        let mut buffer = setup_buffer();
        buffer.lines = vec!(Line::new(String::new(), 0));
        buffer.join_line_with_previous(0, 1);
    }

}

