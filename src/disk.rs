use std::{fs::File, io};

pub const PAGE_SIZE: usize = 4096;

pub struct DiskManager {
    heap_file: File,
    next_page_id: u64,
}

impl DiskManager {
    pub fn new(heap_file: File) -> io::Result<Self> {
       let heap_file_size = heap_file.metadata()?.len();
       let next_page_id = heap_file_size / PAGE_SIZE as u64;
       Ok(Self {
        heap_file,
        next_page_id,
       })
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::{OpenOptions, remove_file}, io::{Write, Read, Seek}};

    use super::DiskManager;

    #[test]
    fn test_disk_manager_new() {
        let file_name = "test.txt";
        let mut file = OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open(file_name)
            .unwrap();
        file.write_all(b"Hello, World!").unwrap();

        let mut disk_manager = DiskManager::new(file).unwrap();

        let mut contents = String::new();
        disk_manager.heap_file.seek(std::io::SeekFrom::Start(0)).unwrap();
        disk_manager.heap_file.read_to_string(&mut contents).unwrap();

        assert_eq!(contents, "Hello, World!");
        assert_eq!(disk_manager.next_page_id, 0);

        remove_file(file_name).unwrap();
    }
}