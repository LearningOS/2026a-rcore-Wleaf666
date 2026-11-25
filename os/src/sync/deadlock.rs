use alloc::vec;
use alloc::vec::Vec;

///死锁状态结构体
pub struct DeadlockState {
    ///是否启用
    pub is_enabled: bool,
    ///可用的资源数量
    pub available: Vec<u32>,
    ///各个线程还需要的资源数量
    pub need: Vec<Vec<u32>>,
    ///各个线程所分配的线程数量
    pub allocation: Vec<Vec<u32>>,
}

impl DeadlockState {
    ///初始化
    pub fn new() -> Self {
        DeadlockState {
            is_enabled: (false),
            available: (Vec::new()),
            need: (Vec::new()),
            allocation: (Vec::new()),
        }
    }
    ///为一个资源分配
    pub fn reigster_resources(&mut self, id: usize, initial: u32) {
        let need_len = id + 1;
        if need_len > self.available.len() {
            self.available.resize(need_len, 0);
            for row in self.need.iter_mut().chain(self.allocation.iter_mut()) {
                row.resize(need_len, 0);
            }
        }
        self.available[id] = initial;
    }
    ///分配线程
    pub fn register_threads(&mut self, tid: usize) {
        let need_len = tid + 1;
        if need_len > self.need.len() {
            self.need.resize(need_len, vec![0; self.available.len()]);
            self.allocation
                .resize(need_len, vec![0; self.available.len()]);
        }
    }

    ///semaphore_id转化
    pub fn semaphore_id2resource_id(semaphore_id:usize)->usize{
        semaphore_id*2+1
    }
    
    ///mutex_id转化
    pub fn mutex_id2resouce_id(mutexid: usize) -> usize {
        mutexid * 2
    }
    ///检测是否死锁了
    pub fn is_deadlocked(&self) -> bool {
        let resource_len = self.available.len();
        let threads_len = self.need.len();
        let mut work = self.available.clone();
        let mut finish = vec![false; threads_len];

        loop {
            let mut found = false;
            for i in 0..threads_len {
                if finish[i] == false && (0..resource_len).all(|j| self.need[i][j] <= work[j]) {
                    for j in 0..resource_len {
                        work[j] += self.allocation[i][j];
                    }
                    finish[i] = true;
                    found = true;
                }
            }

            if !found {
                break;
            }
        }
        finish.iter().any(|&is_finish| !is_finish)
    }
    ///尝试获取锁
    pub fn try_lock(&mut self, tid: usize, resouce_id: usize) -> bool {
        if resouce_id >= self.available.len() {
            return false;
        }

        if tid + 1 > self.need.len() {
            self.register_threads(tid);
        }
        self.need[tid][resouce_id] += 1;

        if self.is_enabled && self.is_deadlocked() {
            self.need[tid][resouce_id] -= 1;
            false
        } else {
            true
        }
    }
    ///锁后的一些操作
    pub fn locked(&mut self, tid: usize, resource_id: usize) {
        self.need[tid][resource_id] -= 1;
        self.available[resource_id] -= 1;
        self.allocation[tid][resource_id] += 1;
    }
    ///解锁后的一些操作
    pub fn unlocked(&mut self, tid: usize, resource_id: usize) {
        self.available[resource_id] += 1;
        self.allocation[tid][resource_id] -= 1
    }
}
