use std::{
    fs::{File, OpenOptions},
    io::{self, Read, Seek, Write},
    path::Path,
};

pub const PAGE_SIZE: usize = 4096;

#[derive(Eq, PartialEq)]
pub struct PageId(pub u64);
impl PageId {
    pub const INVALID_PAGE_ID: PageId = PageId(u64::MAX);

    pub fn valid(self) -> Option<PageId> {
        if self == Self::INVALID_PAGE_ID {
            None
        } else {
            Some(self)
        }
    }

    pub fn to_u64(self) -> u64 {
        self.0
    }
}

impl Default for PageId {
    fn default() -> Self {
        Self::INVALID_PAGE_ID
    }
}

impl From<Option<PageId>> for PageId {
    fn from(page_id: Option<PageId>) -> Self {
        page_id.unwrap_or_default()
    }
}

impl From<&[u8]> for PageId {
    fn from(bytes: &[u8]) -> Self {
        let arr = bytes.try_into().unwrap();
        PageId(u64::from_ne_bytes(arr))
    }
}

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

    pub fn open(heap_file_path: impl AsRef<Path>) -> io::Result<Self> {
        let heap_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(heap_file_path)?;
        Self::new(heap_file)
    }

    pub fn read_page_data(&mut self, page_id: PageId, data: &mut [u8]) -> io::Result<()> {
        let offset = PAGE_SIZE as u64 * page_id.to_u64();
        self.heap_file.seek(io::SeekFrom::Start(offset))?;
        self.heap_file.read_exact(data)
    }

    pub fn write_page_data(&mut self, page_id: PageId, data: &[u8]) -> io::Result<()> {
        let offset = PAGE_SIZE as u64 * page_id.to_u64();
        self.heap_file.seek(io::SeekFrom::Start(offset))?;
        self.heap_file.write_all(data)
    }

    pub fn allocate_page(&mut self) -> PageId {
        let page_id = self.next_page_id;
        self.next_page_id += 1;
        PageId(page_id)
    }

    pub fn sync(&mut self) -> io::Result<()> {
        self.heap_file.flush()?;
        self.heap_file.sync_all()
    }
}

#[cfg(test)]
mod tests {
    use super::DiskManager;

    mod disk_manager {
        use std::{
            fs::{remove_file, File, OpenOptions},
            io::{Read, Seek, Write},
        };

        use crate::disk::PageId;

        use super::DiskManager;

        #[test]
        fn test_disk_manager_new() {
            let file_name = "test_disk_manager_new.txt";
            let file = create_tmp_file(file_name, b"Hello, World!");

            let mut disk_manager = DiskManager::new(file).unwrap();

            let mut contents = String::new();
            disk_manager
                .heap_file
                .seek(std::io::SeekFrom::Start(0))
                .unwrap();
            disk_manager
                .heap_file
                .read_to_string(&mut contents)
                .unwrap();

            assert_eq!(contents, "Hello, World!");
            assert_eq!(disk_manager.next_page_id, 0);

            remove_file(file_name).unwrap();
        }

        #[test]
        fn test_disk_manager_open() {
            let file_name = "test_disk_manager_open.txt";
            create_tmp_file(file_name, b"Hello, World!");

            let mut disk_manager = DiskManager::open(file_name).unwrap();

            let mut contents = String::new();
            disk_manager
                .heap_file
                .seek(std::io::SeekFrom::Start(0))
                .unwrap();
            disk_manager
                .heap_file
                .read_to_string(&mut contents)
                .unwrap();
            assert_eq!(contents, "Hello, World!");
            assert_eq!(disk_manager.next_page_id, 0);

            remove_file(file_name).unwrap();
        }

        #[test]
        fn test_disk_manager_read_page_data() {
            let file_name = "test_disk_manager_read_page_data.txt";
            create_tmp_file(file_name, b"Hello, World!");

            let mut disk_manager = DiskManager::open(file_name).unwrap();
            let page_id = PageId(0);
            // len("Hello, World!") = 13
            let mut buf = vec![0; 13];

            disk_manager.read_page_data(page_id, &mut buf).unwrap();

            assert_eq!(buf, b"Hello, World!");

            remove_file(file_name).unwrap();
        }

        #[test]
        fn test_disk_manager_write_page_data() {
            let file_name = "test_disk_manager_write_page_data.txt";

            let mut disk_manager = DiskManager::open(file_name).unwrap();
            let page_id = PageId(0);
            let buf = b"Hello, World!";

            disk_manager.write_page_data(page_id, buf).unwrap();

            let mut contents = String::new();
            disk_manager
                .heap_file
                .seek(std::io::SeekFrom::Start(0))
                .unwrap();
            disk_manager
                .heap_file
                .read_to_string(&mut contents)
                .unwrap();

            assert_eq!(contents, "Hello, World!");

            remove_file(file_name).unwrap();
        }

        #[test]
        fn test_disk_manager_allocate_page() {
            let file_name = "test_disk_manager_write_page_data.txt";

            let mut disk_manager = DiskManager::open(file_name).unwrap();

            assert_eq!(disk_manager.next_page_id, 0);
            disk_manager.allocate_page();
            assert_eq!(disk_manager.next_page_id, 1);

            remove_file(file_name).unwrap();
        }

        fn create_tmp_file(file_name: &str, contents: &[u8]) -> File {
            let mut file = OpenOptions::new()
                .write(true)
                .read(true)
                .create(true)
                .open(file_name)
                .unwrap();
            file.write_all(contents).unwrap();
            return file;
        }
    }
}
