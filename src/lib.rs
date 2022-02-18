use std::sync::Arc;
use std::thread;
use std::marker:: {Send, Sync};
use std::thread::JoinHandle;

static THRESHOLD: usize = 10;

pub fn transform<T, R, F>(objects: Vec<T>, f: F) -> Vec<R> where
    F: Fn(&T) -> R + Send + Sync + 'static,
    R: Send + 'static,
    T: Send + 'static,
{
    if objects.len() < THRESHOLD {
        objects.iter().map(f).collect()
    } else {
        par_transform(objects, Arc::new(f))
    }
}

fn par_transform<T, R, F>(objects: Vec<T>, f: Arc<F>) -> Vec<R> where
    F: Fn(&T) -> R + Send + Sync + 'static,
    R: Send + 'static,
    T: Send + 'static,
{
    let mut handles: Vec<JoinHandle<R>> = Vec::with_capacity(objects.len());

    for object in objects {
        let f = Arc::clone(&f);

        handles.push(
            thread::spawn(move || {
                f(&object)
            })
        );
    }

    let mut result = vec![];

    for handle in handles {
        result.push(handle.join().unwrap());
    }

    result
}

#[cfg(test)]
mod tests {
    use std::{sync::Mutex, time::Duration};
    use super::*;

    #[test]
    fn less_than_threshold_result() {
        let input = vec!["o", "as", "o", "o", "as", "o", "o", "as", "o",];
        let func = |str: &&str| {
            thread::sleep(Duration::from_secs(1));
            str.len()
        };
        let expected: Vec<usize> = vec![1, 2, 1, 1, 2, 1, 1, 2, 1];

        let result = transform(input, func);

        assert_eq!(result, expected);
    }

    #[test]
    fn greater_than_threshold_result() {
        let input = vec!["o", "as", "o", "o", "as", "o", "o", "as", "o", "o",];
        let func = |str: &&str| {
            thread::sleep(Duration::from_secs(1));
            str.len()
        };
        let expected: Vec<usize> = vec![1, 2, 1, 1, 2, 1, 1, 2, 1, 1];

        let result = transform(input, func);

        assert_eq!(result, expected);
    }

    #[test]
    fn less_than_threshold_is_not_parallel() {
        let input = vec!["o", "as", "o", "o", "as", "o", "o", "as", "o",];
        let thread = thread::current();
        let threads_vec= Arc::new(Mutex::new(vec![]));
        let threads_vec_2 = Arc::clone(&threads_vec);
        let func = move |str: &&str| {
            threads_vec_2.lock().unwrap().push(thread::current());
            thread::sleep(Duration::from_secs(1));
            str.len()
        };

        transform(input, func);

        assert!(threads_vec.lock().unwrap().iter()
            .fold(true, |result, iter_thread| result && iter_thread.name() == thread.name()));
    }

    #[test]
    fn greater_than_threshold_is_parallel() {
        let input = vec!["o", "as", "o", "o", "as", "o", "o", "as", "o", "o",];
        let thread = thread::current();
        let threads_vec= Arc::new(Mutex::new(vec![]));
        let threads_vec_2 = Arc::clone(&threads_vec);
        let func = move |str: &&str| {
            threads_vec_2.lock().unwrap().push(thread::current());
            thread::sleep(Duration::from_secs(1));
            str.len()
        };

        transform(input, func);

        assert!(threads_vec.lock().unwrap().iter()
            .fold(true, |result, iter_thread| result && iter_thread.name() != thread.name()));
    }
}