use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
    convert::TryInto,
    io::{self, Read, Write},
    iter::FromIterator,
};

use crate::histogram::Histogram;
use bit_utils::{read::BitReader, write::WordWriter};

pub struct HuffmanTree {
    word_size: u8,
    nodes: Vec<(usize, usize)>,
    dictionary: HashMap<u64, (u64, usize)>,
}

impl HuffmanTree {
    pub fn encode_word(&self, word: u64) -> (u64, usize) {
        self.dictionary.get(&word).unwrap().clone()
    }

    pub fn decode_next_word(&self, reader: &mut BitReader<impl Read>) -> Result<u64, io::Error> {
        let mut head = 0;

        while head < self.nodes.len() {
            let bit = reader.read(1)?;

            if bit == 0 {
                head = self.nodes[head].0;
            } else {
                head = self.nodes[head].1;
            }
        }

        Ok((head - self.nodes.len()).try_into().unwrap())
    }

    fn compute_dictionary_from_nodes(&mut self) {
        let mut dictionary = HashMap::new();

        let mut queue = vec![(0, vec![])];

        while let Some((id, path)) = queue.pop() {
            let (left, right) = self.nodes[id];

            let mut left_path = path.clone();
            left_path.push(false);
            if left >= self.nodes.len() {
                let converted_value = left_path.iter().fold(0, |a, i| (a << 1) | (*i as u64));
                dictionary.insert(
                    (left - self.nodes.len()) as u64,
                    (converted_value, left_path.len()),
                );
            } else {
                queue.push((left, left_path));
            }

            let mut right_path = path.clone();
            right_path.push(true);
            if right >= self.nodes.len() {
                let converted_value = right_path.iter().fold(0, |a, i| (a << 1) | (*i as u64));
                dictionary.insert(
                    (right - self.nodes.len()) as u64,
                    (converted_value, right_path.len()),
                );
            } else {
                queue.push((right, right_path));
            }
        }

        self.dictionary = dictionary;
    }

    pub fn write(&self, writer: &mut WordWriter<impl Write>) -> Result<(), io::Error> {
        writer.write(((self.word_size - 2) as u64, 4))?; // we allow word sizes 2-17

        let mut queue = vec![self.nodes[0].1, self.nodes[0].0];

        while let Some(item) = queue.pop() {
            if item >= self.nodes.len() {
                writer.write((1, 1))?;
                writer.write(((item - self.nodes.len()) as u64, self.word_size as usize))?;
            } else {
                let (left, right) = self.nodes[item];
                writer.write((0, 1))?;
                queue.push(right);
                queue.push(left);
            }
        }

        Ok(())
    }

    pub fn read(reader: &mut BitReader<impl Read>) -> Result<Self, io::Error> {
        let word_size = reader.read(4)? as usize + 2;

        let mut nodes = vec![(0, 0); (1usize << word_size) - 1];

        let mut path = vec![0];

        let mut free_node = 0;

        while let Some(node) = path.pop() {
            let is_leaf = reader.read(1)?;

            if is_leaf == 1 {
                let word = nodes.len() as u64 + reader.read(word_size)?;

                if nodes[node].0 == 0 {
                    nodes[node].0 = word as usize;
                    path.push(node);
                } else if nodes[node].1 == 0 {
                    nodes[node].1 = word as usize;
                } else {
                    panic!("Unexpected error occurred");
                }
            } else {
                free_node += 1;

                if nodes[node].0 == 0 {
                    nodes[node].0 = free_node;
                    path.push(node);
                } else if nodes[node].1 == 0 {
                    nodes[node].1 = free_node;
                } else {
                    panic!("Unexpected error occurred");
                }

                path.push(free_node);
            }
        }

        let mut output = Self {
            word_size: word_size as u8,
            dictionary: HashMap::new(),
            nodes,
        };

        output.compute_dictionary_from_nodes();

        Ok(output)
    }

    pub fn get_word_size(&self) -> u8 {
        self.word_size
    }
}

impl From<Histogram> for HuffmanTree {
    fn from(value: Histogram) -> Self {
        let word_size = value.get_word_size();
        let histogram = value.get_freq();
        let mut tree = vec![usize::MAX; histogram.len() * 2 - 1];

        #[derive(Debug)]
        struct Node {
            freq: u64,
            index: usize,
        }

        impl PartialEq for Node {
            fn eq(&self, other: &Self) -> bool {
                self.index == other.index
            }
        }

        impl Eq for Node {}

        impl PartialOrd for Node {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                self.freq.partial_cmp(&other.freq)
            }
        }

        impl Ord for Node {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.freq.cmp(&other.freq)
            }
        }

        let mut heap = BinaryHeap::from_iter(histogram.iter().enumerate().map(|(word, &freq)| {
            Reverse(Node {
                index: word,
                freq: freq,
            })
        }).collect::<Vec<_>>());
        let mut free_space = histogram.len();

        loop {
            let Reverse(first) = heap.pop().unwrap();
            let Some(Reverse(second)) = heap.pop() else {
                break;
            };

            let new_freq = first.freq + second.freq;
            let new_index = free_space;
            free_space += 1;
            tree[first.index] = new_index;
            tree[second.index] = new_index;
            heap.push(Reverse(Node {
                freq: new_freq,
                index: new_index,
            }));
        }

        // encode huffman tree as binary tree
        let join_count = histogram.len() - 1;
        let mut tree_2 = vec![(0, 0); join_count];
        let full_tree_length = tree.len();

        // encode all leafs
        for (current, parent) in tree.iter().copied().enumerate().take(histogram.len()) {
            let (left, right) = &mut tree_2[full_tree_length - 1 - parent];

            if *left == 0 {
                *left = join_count + current;
            } else if *right == 0 {
                *right = join_count + current;
            } else {
                debug_assert!(false, "Something wrong happen");
            }
        }

        // encode all parent nodes, except root node
        for (current, parent) in tree
            .into_iter()
            .skip(histogram.len())
            .take(join_count - 1)
            .enumerate()
            .rev()
        {
            let (left, right) = &mut tree_2[full_tree_length - 1 - parent];

            if *left == 0 {
                *left = join_count - current - 1;
            } else if *right == 0 {
                *right = join_count - current - 1;
            } else {
                debug_assert!(false, "Something wrong happen");
            }
        }

        let mut output = HuffmanTree {
            nodes: tree_2,
            dictionary: HashMap::new(),
            word_size,
        };

        output.compute_dictionary_from_nodes();

        output
    }
}

#[cfg(test)]
mod test {
    use std::{collections::HashMap, convert::TryInto};

    use crate::{histogram::Histogram, tree::HuffmanTree};
    use bit_utils::{read::BitReader, write::WordWriter};

    #[test]
    fn should_correctly_build_tree_from_histogram() {
        let histogram: Histogram = vec![5, 9, 5, 5].try_into().unwrap();
        let tree = HuffmanTree::from(histogram);
        assert_eq!(tree.nodes, vec![(1, 2), (4, 6), (3, 5)]);
        assert_eq!(
            tree.dictionary,
            HashMap::from([
                (0b00, (0b10, 2)),
                (0b01, (0b00, 2)),
                (0b10, (0b11, 2)),
                (0b11, (0b01, 2)),
            ])
        );
    }

    #[test]
    fn should_correctly_build_tree_from_histogram_2() {
        let histogram: Histogram = vec![1, 1, 2, 4].try_into().unwrap();
        let tree = HuffmanTree::from(histogram);
        assert_eq!(tree.nodes, vec![(6, 1), (5, 2), (3, 4)]);
        assert_eq!(
            tree.dictionary,
            HashMap::from([
                (0b00, (0b110, 3)),
                (0b01, (0b111, 3)),
                (0b10, (0b10, 2)),
                (0b11, (0b0, 1)),
            ])
        );
    }

    #[test]
    fn should_correctly_build_tree_from_histogram_3() {
        let histogram: Histogram = vec![1, 1, 1, 1, 1, 1, 1, 1].try_into().unwrap();
        let tree = HuffmanTree::from(histogram);
        assert_eq!(
            tree.nodes,
            vec![(1, 2), (3, 5), (4, 6), (10, 11), (8, 14), (12, 13), (7, 9),]
        );
    }

    #[test]
    fn should_correctly_build_tree_from_histogram_4() {
        let histogram: Histogram = vec![1, 1, 2, 4, 8, 16, 32, 64].try_into().unwrap();
        let tree = HuffmanTree::from(histogram);
        assert_eq!(
            tree.nodes,
            vec![(14, 1), (13, 2), (12, 3), (11, 4), (10, 5), (9, 6), (7, 8),]
        );
    }

    #[test]
    fn should_correctly_write_tree() {
        let mut tree = HuffmanTree {
            dictionary: HashMap::new(),
            word_size: 3,
            nodes: vec![(14, 1), (13, 2), (12, 3), (11, 4), (10, 5), (9, 6), (7, 8)],
        };

        tree.compute_dictionary_from_nodes();

        assert_eq!(tree.dictionary, HashMap::new());

        let mut buffer = Vec::new();
        let mut writer: WordWriter<_> = (&mut buffer).into();
        tree.write(&mut writer).unwrap();
        drop(writer);
        assert_eq!(
            buffer,
            vec![0b00011111, 0b01110011, 0b01011000, 0b10110101, 0b00100010, 0b01000000]
        );
    }

    #[test]
    fn should_correctly_write_balanced_tree() {
        let tree = HuffmanTree {
            dictionary: HashMap::new(),
            word_size: 3,
            nodes: vec![(1, 2), (3, 5), (4, 6), (10, 11), (8, 14), (12, 13), (7, 9)],
        };

        let mut buffer = Vec::new();
        let mut writer: WordWriter<_> = (&mut buffer).into();
        tree.write(&mut writer).unwrap();
        drop(writer);
        assert_eq!(
            buffer,
            vec![0b00010010, 0b11110001, 0b10111100, 0b01001111, 0b10100010, 0b10000000]
        );
    }

    #[test]
    fn should_correctly_read_tree() {
        let buffer = vec![
            0b00011111, 0b01110011, 0b01011000, 0b10110101, 0b00100010, 0b01000000,
        ];
        let mut reader = BitReader::new(&buffer[..]);

        let tree = HuffmanTree::read(&mut reader).unwrap();

        assert_eq!(
            tree.nodes,
            vec![(14, 1), (13, 2), (12, 3), (11, 4), (10, 5), (9, 6), (7, 8)],
        );
        assert_eq!(
            tree.dictionary,
            HashMap::from([
                (0b000, (0b1111110, 7)),
                (0b001, (0b1111111, 7)),
                (0b010, (0b111110, 6)),
                (0b011, (0b11110, 5)),
                (0b100, (0b1110, 4)),
                (0b101, (0b110, 3)),
                (0b110, (0b10, 2)),
                (0b111, (0b0, 1)),
            ])
        );
    }

    #[test]
    fn should_correctly_read_balanced_tree() {
        let buffer = vec![
            0b00010010, 0b11110001, 0b10111100, 0b01001111, 0b10100010, 0b10000000,
        ];
        let mut reader = BitReader::new(&buffer[..]);

        let tree = HuffmanTree::read(&mut reader).unwrap();

        assert_eq!(
            tree.nodes,
            vec![(1, 4), (2, 3), (10, 11), (12, 13), (5, 6), (8, 14), (7, 9)],
        );

        assert_eq!(
            tree.dictionary,
            HashMap::from([
                (0b000, (0b110, 3)),
                (0b001, (0b100, 3)),
                (0b010, (0b111, 3)),
                (0b011, (0b000, 3)),
                (0b100, (0b001, 3)),
                (0b101, (0b010, 3)),
                (0b110, (0b011, 3)),
                (0b111, (0b101, 3)),
            ])
        );
    }

    #[test]
    fn tree_serialization_roundtrip() {
        let histogram: Histogram = vec![5, 10, 5, 5].try_into().unwrap();
        let tree: HuffmanTree = histogram.into();

        let mut buffer = Vec::new();
        {
            let mut writer = WordWriter::new(&mut buffer);
            tree.write(&mut writer).unwrap();
        }

        let mut reader = BitReader::new(&buffer[..]);
        let tree_2 = HuffmanTree::read(&mut reader).unwrap();

        assert_eq!(tree.dictionary, tree_2.dictionary);
    }

    #[test]
    fn tree_serialization_roundtrip_2() {
        let histogram: Histogram = vec![4, 2, 2, 1, 1, 1, 1, 1].try_into().unwrap();
        let tree: HuffmanTree = histogram.into();

        let mut buffer = Vec::new();
        {
            let mut writer = WordWriter::new(&mut buffer);
            tree.write(&mut writer).unwrap();
        }

        let mut reader = BitReader::new(&buffer[..]);
        let tree_2 = HuffmanTree::read(&mut reader).unwrap();

        assert_eq!(tree.dictionary, tree_2.dictionary);
    }
}
