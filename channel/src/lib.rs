use std::{
    collections::VecDeque,
    mem,
    sync::{Arc, Condvar, Mutex},
};

pub struct Sender<T> {
    shared: Arc<Shared<T>>,
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        let mut inner = self.shared.inner.lock().unwrap();
        inner.sender += 1;
        drop(inner);
        Sender {
            shared: Arc::clone(&self.shared),
        }
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        let mut inner = self.shared.inner.lock().unwrap();
        inner.sender -= 1;
        let was_last = inner.sender == 0;
        drop(inner);
        if was_last {
            self.shared.available.notify_one();
        }
    }
}

impl<T> Sender<T> {
    fn send(&mut self, t: T) {
        let mut inner = self.shared.inner.lock().unwrap();
        inner.queue.push_back(t);
        drop(inner);
        self.shared.available.notify_one();
    }
}

impl<T> Receiver<T> {
    fn receive(&mut self) -> Option<T> {
        if let Some(value) = self.buffer.pop_front() {
            return Some(value);
        }
        let mut inner = self.shared.inner.lock().unwrap();
        loop {
            match inner.queue.pop_front() {
                Some(t) => {
                    if !inner.queue.is_empty() {
                        mem::swap(&mut inner.queue, &mut self.buffer);
                    }
                    return Some(t);
                }
                None if inner.sender == 0 => return None,
                None => {
                    inner = self.shared.available.wait(inner).unwrap();
                }
            }
        }
    }
}
pub struct Receiver<T> {
    shared: Arc<Shared<T>>,
    buffer: VecDeque<T>,
}

struct Inner<T> {
    queue: VecDeque<T>,
    sender: usize,
}

struct Shared<T> {
    inner: Mutex<Inner<T>>,
    available: Condvar,
}

fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let inner = Inner {
        queue: VecDeque::default(),
        sender: 1,
    };
    let shared = Shared {
        inner: Mutex::new(inner),
        available: Condvar::new(),
    };

    let shared = Arc::new(shared);

    (
        Sender {
            shared: shared.clone(),
        },
        Receiver {
            shared: shared.clone(),
            buffer: VecDeque::default(),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_ping_pong() {
        let (mut sender, mut receiver) = channel();
        sender.send(100);

        assert_eq!(receiver.receive().unwrap(), 100);
    }

    #[test]
    fn test_with_sender_closed() {
        let (sender, mut receiver) = channel::<()>();
        drop(sender);

        // Waits forever has sender cannot send any signal.
        assert_eq!(receiver.receive(), None);
    }

    #[test]
    fn test_with_receiver_closed() {
        let (mut sender, receiver) = channel();
        drop(receiver);

        // Should not send when receiver is closed.
        sender.send(100)
    }
}
