use futures_util::future::join_all;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::future::Future;
use std::io::{prelude::*, BufReader, SeekFrom};
use std::pin::Pin;
use std::sync::{Arc, Mutex, MutexGuard};
use std::task::{Context, Poll};
use tokio::task::JoinHandle;

type AsyncFileHandle = Arc<Mutex<File>>;
type FileJoinHandle = JoinHandle<Result<String, String>>;

#[derive(Debug)]
enum FileError {
    ReadError(String),
    SeekError(String)
}

fn get_handle(file_path: &dyn ToString) -> AsyncFileHandle {
    let file = OpenOptions::new()
        .append(true)
        .read(true)
        .write(true)
        .open(file_path.to_string())
        .or_else(|_| File::create(file_path.to_string()))
        .expect("Failed to open file");

    return Arc::new(Mutex::new(file));
}

struct AsyncReadFutue {
    pub handle: AsyncFileHandle,
}

impl AsyncReadFutue {

    fn poll_file_lock(&self, cx: &mut Context<'_>) -> Option<MutexGuard<'_, File>> {
        match self.handle.try_lock() {
            Ok(guard) => Some(guard),
            Err(_) => {
                cx.waker().wake_by_ref();
                None
            }
        }
    }

    fn read_file_by_reader(&self, file: MutexGuard<'_, File>) -> String {
        let mut reader = BufReader::new(& *file);
        let mut buffer = String::new();
        let mut line = String::new();

        while let Ok(bytes_read) = reader.read_line(&mut line) {
            if bytes_read == 0 {
                break;
            }

            buffer.push_str(&line);
            line.clear();
        }

        buffer
    }

    /// Reads the file content from the beginning to the end.
    /// 
    /// This method first resets the file pointer to the start of the file,
    /// then attempts to read the file content into a string. If an error occurs
    /// during the read process, it returns a `FileError` type error, with the
    /// specific error information logged to the console. If the read is successful,
    /// it returns the string of the file content read.
    fn read_file_seek(&self, file: &mut MutexGuard<'_, File>) -> Result<String, FileError> {
        // 此处有一个问题：
        // 如果是写完后立即读取时，如果使用File::read_to_string方法，可能会导致文件读取不完整，因为会从文件的当前位置开始读取，而不是从文件开头开始读取。
        let mut buffer = String::new();

        match file.seek(SeekFrom::Start(0)) {
            Ok(_) => {},
            Err(err) => {
                let msg = format!("SeekFrom to start of file error: {}", err);
                println!("[将文件指针重置到文件开头] error: {}", msg);
                return Err(FileError::SeekError(msg.into()));
            }
        }

        match file.read_to_string(&mut buffer) {
            Ok(_bytes_read) => {
                Ok(buffer)
            }
            Err(err) => {
                let es = format!("Error reading from file: {}", err);
                Err(FileError::ReadError(es.into()))
            }
        }
    }

}

impl Future for AsyncReadFutue {
    type Output = Result<String, String>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let guard = self.poll_file_lock(cx);
        if guard.is_none() {
            cx.waker().wake_by_ref();
            return Poll::Pending;
        }

        let mut file = guard.unwrap();
        
        // 方法一
        // match self.read_file_seek(&mut file) {
        //     Ok(content) => Poll::Ready(Ok(content)),
        //     Err(e) => {
        //         println!("{:?}", e);
                
        //         match e {
        //             FileError::ReadError(msg) => {
        //                 Poll::Ready(Err(msg))
        //             },
        //             FileError::SeekError(msg) => {
        //                 println!("[将文件指针重置到文件开头]: {}", msg);
        //                 cx.waker().wake_by_ref();
        //                 return Poll::Pending;
        //             },
        //             _ => {
        //                 cx.waker().wake_by_ref();
        //                 return Poll::Pending;
        //             }
        //         }
        //     },
        // }
        // ------------------------------------------------
        
        // 方法二
        Poll::Ready(Ok(self.read_file_by_reader(file)))
    }
}

fn read_file(read_handle: AsyncFileHandle) -> FileJoinHandle {
    let read = AsyncReadFutue {
        handle: read_handle,
    };

    tokio::task::spawn(async move { read.await })
}

struct AsyncWriteFuture {
    pub handle: AsyncFileHandle,
    pub entry: String,
}

impl Future for AsyncWriteFuture {
    type Output = Result<String, String>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut guard = match self.handle.try_lock() {
            Ok(guard) => guard,
            Err(err) => {
                println!("try_lock Error for {}: {}", self.entry, err);
                cx.waker().wake_by_ref();
                return Poll::Pending;
            }
        };

        let line_entry = format!("{}\n", self.entry);
        match guard.write_all(line_entry.as_bytes()) {
            Ok(_) => println!("written content: {}", self.entry),
            Err(e) => println!("write error: {}", e),
        }

        Poll::Ready(Ok(String::new()))
    }
}

fn write_log(file_handle: AsyncFileHandle, entry: String) -> FileJoinHandle {
    let write = AsyncWriteFuture {
        entry: entry,
        handle: file_handle,
    };

    tokio::task::spawn(async move { write.await })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let login_handle = get_handle(&"login.txt");
    let logout_handle = get_handle(&"logout.txt");

    let names = vec!["Alice", "Bob", "Charlie"];
    let mut handles = vec![];

    for name in names {
        let login_entry = format!("{} logged in\n", name);
        let logout_entry = format!("{} logged out\n", name);

        let handle_one = write_log(login_handle.clone(), login_entry);
        let handle_two = write_log(logout_handle.clone(), logout_entry);

        handles.push(handle_one);
        handles.push(handle_two);
    }

    let _ = join_all(handles).await;


    // println!("--------------------------------------------");
    // let result = read_file(login_handle).await?;
    // println!("{:?}", result);

    Ok(())
}
