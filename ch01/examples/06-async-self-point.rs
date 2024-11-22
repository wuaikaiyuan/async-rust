use std::ptr;

// 自引用

struct SelfReferential {
    data: String,
    // 指向 data 的指针
    self_pointer: *const String,
}

impl SelfReferential {
    fn new(data: String) -> Self {
        let mut sr = SelfReferential {
            data,
            self_pointer: ptr::null(),
        };

        sr.self_pointer = &sr.data as *const String;
        sr
    }

    fn print(&self) {
        unsafe {
            println!("self_pointer: {}", *self.self_pointer);
        }
    }
}

fn main() {
    let sr = SelfReferential::new(String::from("data"));
    // move struct SelfReferential
    let move_sr = sr;
    move_sr.print();
}
