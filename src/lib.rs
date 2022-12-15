// TODO:
// - rollback
// - "life" logic

// Interface
// =========

/// Trait for the user-implemented value wich will be snapshoted.
///
/// The `E` type parameter represents an extra value that the implemetation can
/// return on a layer merge event to be reused outside.
pub trait Layer
where
    Self: Sized,
{
    type Extra;

    fn get_tick(&self) -> u64;

    fn merge(self, other: Self) -> (Self, Self::Extra);
}

// Snapshots
// =========

enum Node<T> {
    End,
    Layer {
        value: T,
        counter: u64,
        tail: Box<Node<T>>,
    },
}

impl<T> Default for Node<T> {
    fn default() -> Self {
        Node::End
    }
}

pub struct Snapshots<T: Layer> {
    ratio: u64,
    tick: u64,
    layers: Node<T>,
}

impl<T: Layer> Snapshots<T> {
    /// * `ratio`: Number snapshots on an upper layer necessary to trigger an
    ///   snapshot on a lower layer. Should be > 0.
    ///
    /// Returns `None` if `ratio` is 0.
    pub fn new(ratio: u64) -> Option<Self> {
        if ratio < 1 {
            return None;
        }
        let layers = Node::End;
        Some(Snapshots {
            ratio,
            tick: 0,
            layers,
        })
    }

    pub fn last(&self) -> Option<&T> {
        match &self.layers {
            Node::End => None,
            Node::Layer {
                value,
                counter: _,
                tail: _,
            } => Some(value),
        }
    }

    pub fn insert(&mut self, value: T) -> Option<T::Extra> {
        let layers = std::mem::take(&mut self.layers);
        let (layers, extra) = do_insert(self.ratio, layers, value);
        *self = Snapshots {
            ratio: self.ratio,
            tick: self.tick + 1,
            layers,
        };
        extra
    }

    // Iteration

    pub fn iter(&self) -> LayerIterator<T> {
        LayerIterator {
            layers: &self.layers,
        }
    }
}

pub struct LayerIterator<'a, T> {
    layers: &'a Node<T>,
}

impl<'a, T> Iterator for LayerIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.layers {
            Node::End => None,
            Node::Layer {
                value,
                counter: _,
                tail,
            } => {
                self.layers = tail;
                Some(value)
            }
        }
    }
}

fn do_insert<T: Layer>(ratio: u64, layers: Node<T>, new_value: T) -> (Node<T>, Option<T::Extra>) {
    let max_counter = ratio - 1;
    match layers {
        // When there's only the initial base layer
        Node::End => {
            let layers = Node::Layer {
                value: new_value,
                counter: 0,
                tail: Box::new(layers),
            };
            (layers, None)
        }
        // When there are non-initial layers
        Node::Layer {
            value: old_value,
            counter,
            tail,
        } => {
            // If we have reached the counter limit on current layer
            if counter >= max_counter {
                // Add new layer for inserted value and merge old value to
                // old layers below
                let (tail, extra) = do_insert(ratio, *tail, old_value);
                let layers = Node::Layer {
                    value: new_value,
                    counter: 0,
                    tail: Box::new(tail),
                };
                (layers, extra)
            } else {
                // Merge new value to current layer and increment counter
                let (merged, extra) = old_value.merge(new_value);
                let new_layers = Node::Layer {
                    value: merged,
                    counter: counter + 1,
                    tail,
                };
                (new_layers, Some(extra))
            }
        }
    }
}

// Tests
// =====

#[cfg(test)]
mod tests {
    use super::*;

    fn show_layers_with_iter<T: Layer>(snaps: &Snapshots<T>, f: impl Fn(&T) -> String) {
        let vals: Vec<_> = snaps.iter().map(f).collect();
        eprint!("*");
        for v in vals.iter().rev() {
            eprint!(" {}", v);
        }
        eprintln!();
    }

    struct NumLayer(u64);

    impl Layer for NumLayer {
        type Extra = ();
        fn get_tick(&self) -> u64 {
            self.0
        }

        fn merge(self, other: Self) -> (Self, Self::Extra) {
            assert!(self.0 < other.0);
            (NumLayer(other.0), ())
        }
    }

    #[test]
    fn it_works() {
        let mut snaps: Snapshots<NumLayer> = Snapshots::new(4).unwrap();
        let mk_next = |curr: &NumLayer| NumLayer(curr.0 + 1);
        for _ in 0..32 {
            let next = snaps.last().map(mk_next).unwrap_or_else(|| NumLayer(1));
            snaps.insert(next);
            show_layers_with_iter(&snaps, |v| format!("{}", v.0));
        }
    }
}
