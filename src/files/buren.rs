use buren::PartialDeserializer;

struct BurenReader<T> {
    deser: PartialDeserializer
}

impl BurenReader<T> {
    pub fn new<O>(f:O, columns: &[usize]) -> PartialDeserializer<R>
        where O: Fn(usize) -> R {
            BurenReader { deser:PartialDeserializer::new(f,columns) }
        }

}
