//! We use this mocking module in unit tests to emulate a serial port.

/// Our mock type used to emulate a serial port.
pub struct MockSerial {
    /// Buffer to store data written to the mock serial port
    write_buffer: heapless::Vec<u8, 256>,
    /// Buffer containing pre-configured response data to be read
    read_buffer: heapless::Vec<u8, 256>,
    /// Current position in the read buffer
    read_position: usize,
    /// Flag to simulate write errors
    should_error_on_write: bool,
    /// Flag to simulate read errors
    should_error_on_read: bool,
}

#[derive(Debug)]
pub enum MockSerialError {
    /// Simulated timeout error
    Timeout,
    /// Simulated buffer overflow
    BufferOverflow,
    /// Simulated invalid data error
    InvalidData,
    /// Generic simulated error for testing
    SimulatedError,
    /// Would block - no data available
    WouldBlock,
}

impl embedded_io::Error for MockSerialError {
    fn kind(&self) -> embedded_io::ErrorKind {
        match self {
            MockSerialError::Timeout => embedded_io::ErrorKind::TimedOut,
            MockSerialError::BufferOverflow => embedded_io::ErrorKind::OutOfMemory,
            MockSerialError::InvalidData => embedded_io::ErrorKind::InvalidData,
            MockSerialError::SimulatedError => embedded_io::ErrorKind::Other,
            MockSerialError::WouldBlock => embedded_io::ErrorKind::Other,
        }
    }
}


// impl 

impl embedded_io::ErrorType for MockSerial {
    type Error = MockSerialError;
}

impl embedded_io::Write for MockSerial {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        if self.should_error_on_write {
            return Err(MockSerialError::SimulatedError);
        }

        let available_space = self.write_buffer.capacity() - self.write_buffer.len();
        if buf.len() > available_space {
            return Err(MockSerialError::BufferOverflow);
        }

        for &byte in buf {
            self.write_buffer.push(byte).map_err(|_| MockSerialError::BufferOverflow)?;
        }

        Ok(buf.len())
    }
    
    fn flush(&mut self) -> Result<(), Self::Error> {
        if self.should_error_on_write {
            return Err(MockSerialError::SimulatedError);
        }
        Ok(())
    }
}

impl embedded_io::Read for MockSerial {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        if self.should_error_on_read {
            return Err(MockSerialError::SimulatedError);
        }

        if self.read_position >= self.read_buffer.len() {
            return Err(MockSerialError::WouldBlock);
        }

        let available_bytes = self.read_buffer.len() - self.read_position;
        let bytes_to_read = core::cmp::min(buf.len(), available_bytes);

        for i in 0..bytes_to_read {
            buf[i] = self.read_buffer[self.read_position + i];
        }

        self.read_position += bytes_to_read;
        Ok(bytes_to_read)
    }
}

impl MockSerial {
    /// Create a new MockSerial instance with empty buffers
    pub fn new() -> Self {
        Self {
            write_buffer: heapless::Vec::new(),
            read_buffer: heapless::Vec::new(),
            read_position: 0,
            should_error_on_write: false,
            should_error_on_read: false,
        }
    }

    /// Set the data that will be returned when read() is called
    pub fn set_read_data(&mut self, data: &[u8]) -> Result<(), MockSerialError> {
        self.read_buffer.clear();
        self.read_position = 0;
        
        for &byte in data {
            self.read_buffer.push(byte).map_err(|_| MockSerialError::BufferOverflow)?;
        }
        
        Ok(())
    }

    /// Get a reference to the data that was written to this mock serial port
    pub fn written_data(&self) -> &[u8] {
        &self.write_buffer
    }

    /// Clear the write buffer
    pub fn clear_written_data(&mut self) {
        self.write_buffer.clear();
    }

    /// Reset the read position to the beginning of the read buffer
    pub fn reset_read_position(&mut self) {
        self.read_position = 0;
    }

    /// Configure whether write operations should fail with an error
    pub fn set_write_error(&mut self, should_error: bool) {
        self.should_error_on_write = should_error;
    }

    /// Configure whether read operations should fail with an error
    pub fn set_read_error(&mut self, should_error: bool) {
        self.should_error_on_read = should_error;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_io::{Read, Write, Error};

    #[test]
    fn test_new_mock_serial() {
        let mock = MockSerial::new();
        assert_eq!(mock.written_data().len(), 0);
        assert_eq!(mock.read_position, 0);
        assert_eq!(mock.should_error_on_write, false);
        assert_eq!(mock.should_error_on_read, false);
    }

    #[test]
    fn test_write_data() {
        let mut mock = MockSerial::new();
        let test_data = b"Hello, World!";
        
        let result = mock.write(test_data);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), test_data.len());
        assert_eq!(mock.written_data(), test_data);
    }

    #[test]
    fn test_write_multiple_times() {
        let mut mock = MockSerial::new();
        let data1 = b"Hello, ";
        let data2 = b"World!";
        
        mock.write(data1).unwrap();
        mock.write(data2).unwrap();
        
        let expected = b"Hello, World!";
        assert_eq!(mock.written_data(), expected);
    }

    #[test]
    fn test_write_buffer_overflow() {
        let mut mock = MockSerial::new();
        let large_data = vec![0u8; 300]; // Larger than 256 byte capacity
        
        let result = mock.write(&large_data);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), MockSerialError::BufferOverflow));
    }

    #[test]
    fn test_flush() {
        let mut mock = MockSerial::new();
        let result = mock.flush();
        assert!(result.is_ok());
    }

    #[test]
    fn test_read_data() {
        let mut mock = MockSerial::new();
        let test_data = b"Response data";
        mock.set_read_data(test_data).unwrap();
        
        let mut buffer = [0u8; 20];
        let result = mock.read(&mut buffer);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), test_data.len());
        assert_eq!(&buffer[..test_data.len()], test_data);
    }

    #[test]
    fn test_read_partial_data() {
        let mut mock = MockSerial::new();
        let test_data = b"Long response data";
        mock.set_read_data(test_data).unwrap();
        
        let mut buffer = [0u8; 5];
        let result = mock.read(&mut buffer);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 5);
        assert_eq!(&buffer, b"Long ");
    }

    #[test]
    fn test_read_multiple_calls() {
        let mut mock = MockSerial::new();
        let test_data = b"Hello World";
        mock.set_read_data(test_data).unwrap();
        
        let mut buffer1 = [0u8; 5];
        let mut buffer2 = [0u8; 6];
        
        let result1 = mock.read(&mut buffer1);
        let result2 = mock.read(&mut buffer2);
        
        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert_eq!(result1.unwrap(), 5);
        assert_eq!(result2.unwrap(), 6);
        assert_eq!(&buffer1, b"Hello");
        assert_eq!(&buffer2, b" World");
    }

    #[test]
    fn test_read_timeout_when_no_data() {
        let mut mock = MockSerial::new();
        let mut buffer = [0u8; 10];
        
        let result = mock.read(&mut buffer);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), MockSerialError::WouldBlock));
    }

    #[test]
    fn test_read_timeout_after_data_exhausted() {
        let mut mock = MockSerial::new();
        let test_data = b"Hi";
        mock.set_read_data(test_data).unwrap();
        
        let mut buffer = [0u8; 10];
        
        // First read should succeed
        let result1 = mock.read(&mut buffer);
        assert!(result1.is_ok());
        
        // Second read should return WouldBlock
        let result2 = mock.read(&mut buffer);
        assert!(result2.is_err());
        assert!(matches!(result2.unwrap_err(), MockSerialError::WouldBlock));
    }

    #[test]
    fn test_write_error_simulation() {
        let mut mock = MockSerial::new();
        mock.set_write_error(true);
        
        let test_data = b"test";
        let result = mock.write(test_data);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), MockSerialError::SimulatedError));
        assert_eq!(mock.written_data().len(), 0); // Nothing should be written
    }

    #[test]
    fn test_flush_error_simulation() {
        let mut mock = MockSerial::new();
        mock.set_write_error(true);
        
        let result = mock.flush();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), MockSerialError::SimulatedError));
    }

    #[test]
    fn test_read_error_simulation() {
        let mut mock = MockSerial::new();
        mock.set_read_data(b"test data").unwrap();
        mock.set_read_error(true);
        
        let mut buffer = [0u8; 10];
        let result = mock.read(&mut buffer);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), MockSerialError::SimulatedError));
    }

    #[test]
    fn test_error_kinds() {
        assert!(matches!(MockSerialError::Timeout.kind(), embedded_io::ErrorKind::TimedOut));
        assert!(matches!(MockSerialError::BufferOverflow.kind(), embedded_io::ErrorKind::OutOfMemory));
        assert!(matches!(MockSerialError::InvalidData.kind(), embedded_io::ErrorKind::InvalidData));
        assert!(matches!(MockSerialError::SimulatedError.kind(), embedded_io::ErrorKind::Other));
    }

    #[test]
    fn test_clear_written_data() {
        let mut mock = MockSerial::new();
        mock.write(b"test data").unwrap();
        assert!(!mock.written_data().is_empty());
        
        mock.clear_written_data();
        assert!(mock.written_data().is_empty());
    }

    #[test]
    fn test_reset_read_position() {
        let mut mock = MockSerial::new();
        let test_data = b"Hello World";
        mock.set_read_data(test_data).unwrap();
        
        let mut buffer = [0u8; 5];
        mock.read(&mut buffer).unwrap(); // Advances read position
        
        mock.reset_read_position();
        
        let result = mock.read(&mut buffer);
        assert!(result.is_ok());
        assert_eq!(&buffer, b"Hello");
    }

    #[test]
    fn test_set_read_data_buffer_overflow() {
        let mut mock = MockSerial::new();
        let large_data = vec![0u8; 300]; // Larger than 256 byte capacity
        
        let result = mock.set_read_data(&large_data);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), MockSerialError::BufferOverflow));
    }

    #[test]
    fn test_set_read_data_clears_previous() {
        let mut mock = MockSerial::new();
        mock.set_read_data(b"first").unwrap();
        mock.set_read_data(b"second").unwrap();
        
        let mut buffer = [0u8; 10];
        let result = mock.read(&mut buffer);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 6);
        assert_eq!(&buffer[..6], b"second");
    }

    #[test]
    fn test_error_flags_toggle() {
        let mut mock = MockSerial::new();
        
        // Test write error flag
        mock.set_write_error(true);
        assert!(mock.write(b"test").is_err());
        
        mock.set_write_error(false);
        assert!(mock.write(b"test").is_ok());
        
        // Test read error flag
        mock.set_read_data(b"data").unwrap();
        mock.set_read_error(true);
        
        let mut buffer = [0u8; 10];
        assert!(mock.read(&mut buffer).is_err());
        
        mock.set_read_error(false);
        mock.reset_read_position();
        assert!(mock.read(&mut buffer).is_ok());
    }
}