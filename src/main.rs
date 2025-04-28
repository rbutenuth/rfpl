use std::{mem, sync::Arc};
use rfpl::{Value};

#[derive(Debug)]
struct Droppable {
    value: u32,
}

impl Drop for Droppable {
    fn drop(&mut self) {
        println!("dropping {}", self.value);
    }
}

fn main() {
    let s = String::from("hello");
    println!("s: {}", s);

    let len = 10;

    let mut uninitialized: Box<[std::mem::MaybeUninit<Droppable>]> = Box::new_uninit_slice(len);

    // geht, wird aber nie gedropped, da überschrieben!
    uninitialized[0].write(Droppable{ value: 42});
    // geht nicht:
    // unitialized[1]= Droppable{ value: 42};

    {
        let mut boxed_slice: Box<[Droppable]> = unsafe { uninitialized.assume_init() };

        for i in 0..len {
            boxed_slice[i].value = i as u32;
        }
        
        println!("{:?}", boxed_slice);

    }

    println!("done with box");

    let mut uninitialized: Arc<[std::mem::MaybeUninit<Droppable>]> = Arc::new_uninit_slice(len);
    // Wenn man eine zweite Referenz erzeugt, knallt es:
    // let mut u2 = Arc::clone(&uninitialized);
    let mutuable = Arc::get_mut(&mut uninitialized).unwrap();

    for i in 0..len {
        mutuable[i].write(Droppable{ value: 42 + i as u32});
    }
    let initialized: Arc<[Droppable]> = unsafe { uninitialized.assume_init() };
    println!("value: {}", initialized[4].value);
    println!("size: {}", mem::size_of::<Arc<Droppable>>());

    println!("strong count: {}, weak count: {}", Arc::strong_count(&initialized), Arc::weak_count(&initialized));

    println!("size of Value: {}", mem::size_of::<Value>());
}
