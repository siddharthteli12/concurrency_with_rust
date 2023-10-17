fn sum_parallel(list: Vec<i32>) -> i32 {
    let thread_num = 4;

    let iter_size = list.len() / thread_num;
    let mut handler = vec![];
    for i in 0..thread_num {
        let start_index = i * iter_size;
        let end_index = if i == thread_num - 1 {
            list.len()
        } else {
            start_index + iter_size
        };

        let chunk = list[start_index..end_index].to_vec();

        let handle = std::thread::spawn(move || {
            let sum = chunk.iter().sum();
            sum
        });
        handler.push(handle);
    }

    let mut result: Vec<i32> = vec![];
    for handle in handler {
        result.push(handle.join().unwrap());
    }
    result.iter().sum()
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn simple_list_1() {
        let list = vec![100, 200, 12, 1213, 12341, 1243, 2345, 53, 645, 45];

        assert_eq!(sum_parallel(list.clone()), list.iter().sum());
    }
}
