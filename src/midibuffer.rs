use std::{collections::VecDeque, ops::Index};

pub struct PolyMidiBuffer {
    pub notes: VecDeque<u8>,
    max_size: usize,
}

impl PolyMidiBuffer {
    pub fn new(size: usize) -> Self {
        PolyMidiBuffer {
            notes: VecDeque::new(),
            max_size: size,
        }
    }

    pub fn add_note(&mut self, midi_note: u8) {
        //if note already exist, exit
        for note in &self.notes {
            if midi_note == *note {
                return;
            }
        }
        
        //if there is place left in the buffer, push the new value, else, pop the oldest
        if self.notes.len() == self.max_size {
            self.notes.pop_front();
        }
        
        self.notes.push_back(midi_note);
    }

    pub fn remove_note(&mut self, midi_note: u8) {
  
        self.notes.retain(|&x| x!=midi_note);
    }

    pub fn kill_all(&mut self){
        self.notes.clear();
    }
}




#[cfg(test)]
mod test {
    use super::PolyMidiBuffer;

    #[test]
    fn basics() {
        let mut buffer = PolyMidiBuffer::new(4);

        //add and remove note
        buffer.add_note(55);
        buffer.remove_note(55);
        
        assert_eq!(buffer.notes.len(), 0);

        //fill the buffer
        for i in 0..4{
            buffer.add_note(40+i)
        }

        assert_eq!(buffer.notes.get(0), Some(40).as_ref());
        assert_eq!(buffer.notes.get(1), Some(41).as_ref());
        assert_eq!(buffer.notes.get(2), Some(42).as_ref());
        assert_eq!(buffer.notes.get(3), Some(43).as_ref());

        //overflow, replace oldest note
        buffer.add_note(50);

        assert_eq!(buffer.notes.get(0), Some(41).as_ref());
        assert_eq!(buffer.notes.get(1), Some(42).as_ref());
        assert_eq!(buffer.notes.get(2), Some(43).as_ref());
        assert_eq!(buffer.notes.get(3), Some(50).as_ref());

        //second overflow, replace oldest note
        buffer.add_note(60);

        assert_eq!(buffer.notes.get(0), Some(42).as_ref());
        assert_eq!(buffer.notes.get(1), Some(43).as_ref());
        assert_eq!(buffer.notes.get(2), Some(50).as_ref());
        assert_eq!(buffer.notes.get(3), Some(60).as_ref());

        //remove a note in the middle
        buffer.remove_note(43);

        assert_eq!(buffer.notes.get(0), Some(42).as_ref());
        assert_eq!(buffer.notes.get(1), Some(50).as_ref());
        assert_eq!(buffer.notes.get(2), Some(60).as_ref());

        //push a new note
        buffer.add_note(55);

        assert_eq!(buffer.notes.get(0), Some(42).as_ref());
        assert_eq!(buffer.notes.get(1), Some(50).as_ref());
        assert_eq!(buffer.notes.get(2), Some(60).as_ref());
        assert_eq!(buffer.notes.get(3), Some(55).as_ref());

    }


}