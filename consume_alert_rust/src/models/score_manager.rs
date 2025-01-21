use crate::common::*;

#[derive(Debug, Clone, Getters)]
#[getset(get = "pub")]
pub struct ScoredData<T> {
    pub score: i32,
    pub data: T,
}

/* Sort by Score */
#[derive(Eq, PartialEq)]
struct MinHeapItem(i32);

impl Ord for MinHeapItem {
    fn cmp(&self, other: &Self) -> Ordering {
        /* Low Score Priority Sorting (min-heap) */
        other.0.cmp(&self.0)
    }
}

impl PartialOrd for MinHeapItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct ScoreManager<T> {
    heap: BinaryHeap<MinHeapItem>, /* Manage your score to a minimum heap */
    data_map: HashMap<i32, Vec<ScoredData<T>>>, /* Data Management by Score */
}

impl<T> ScoreManager<T> {
    pub fn new() -> Self {
        Self {
            heap: BinaryHeap::new(),
            data_map: HashMap::new(),
        }
    }

    /* Insert Score and Data */
    pub fn insert(&mut self, score: i32, data: T) {
        /* Insert Data */
        self.data_map
            .entry(score)
            .or_insert_with(Vec::new)
            .push(ScoredData { score, data });

        /* Add scores to the heap (you can insert duplicate scores that already exist) */
        if !self.heap.iter().any(|MinHeapItem(s)| *s == score) {
            self.heap.push(MinHeapItem(score));
        }
    }

    /* Get the lowest score and data */
    pub fn pop_lowest(&mut self) -> Option<ScoredData<T>> {
        /* Get the lowest score in the heap */
        let lowest_score = self.heap.pop()?.0;

        /* Pull one out of the data list for that score */
        if let Some(mut data_list) = self.data_map.remove(&lowest_score) {
            let result = data_list.pop();

            /* Reinsert data if it remains */
            if !data_list.is_empty() {
                self.data_map.insert(lowest_score, data_list);
                self.heap.push(MinHeapItem(lowest_score)); /* Add score back to heap */
            }

            return result;
        }

        None
    }
}
