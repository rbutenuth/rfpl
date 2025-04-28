use std::sync::Arc;
use std::mem::MaybeUninit;

use crate::Value;

use super::{Bucket, FplList};

const BASE_SIZE: usize = 8;
const FACTOR: usize = 4;


impl FplList {

    pub fn empty() -> FplList {
        FplList{ buckets: unsafe { Arc::new_uninit_slice(0).assume_init() } }
    }

    pub fn len(&self) -> usize {
        self.buckets.iter().map(|b| b.values.len()).sum()
    }

    pub fn from_value(value: Value) -> FplList {
        let mut u_values: Arc<[MaybeUninit<Value>]> = Arc::new_uninit_slice(1);
        let mutuable = Arc::get_mut(&mut u_values).unwrap();
        mutuable[0].write(value);
        let bucket = Bucket{ values: unsafe { u_values.assume_init() }};

        let mut u_buckets: Arc<[MaybeUninit<Bucket>]> = Arc::new_uninit_slice(1);
        let mutuable = Arc::get_mut(&mut u_buckets).unwrap();
        mutuable[0].write(bucket);
        FplList{ buckets: unsafe { u_buckets.assume_init() }}
    }
}

pub fn experiment() {
    let mut values = Box::<[u32]>::new_uninit_slice(3);
    // Deferred initialization:
    values[0].write(1);
    values[1].write(2);
    values[2].write(3);

    let values = unsafe { values.assume_init() };

    println!("value[2]: {}", values[2]);
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_has_size_0() {
        assert_eq!(0, FplList::empty().len());
    }
    #[test]
    fn test_from_value_has_size_1() {
        let list = FplList::from_value(Value::Integer(42));
        assert_eq!(1, list.len());
        match list.buckets[0].values[0].clone() {
            Value::Integer(i) => assert_eq!(42, i),
            _ => panic!("should be Integer"),
        }
    }
    #[test]
    fn test_clone() {
        let list = FplList::from_value(Value::Integer(42));
        assert_eq!(1, Arc::strong_count(&(list.buckets)));
        let cloned = list.clone();
        assert_eq!(list.len(), cloned.len());
        assert_eq!(2, Arc::strong_count(&(list.buckets)));
    }
}