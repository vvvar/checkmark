/// Represents the context of a list in a markdown document.
#[derive(Debug)]
pub struct ListContext {
    pub nesting_level: usize,
    pub is_ordered: bool,
    pub num_item: u32,
    pub spread: bool,
}

/// Represents the context of a block quote in a markdown document.
#[derive(Debug)]
pub struct BlockQuoteContext {
    pub depth: usize,
}

/// Represents the context of a block quote within a list in a markdown document.
#[derive(Debug)]
pub struct BlockQuoteInListContext {
    pub list_ctx: ListContext,
    pub block_quote_ctx: BlockQuoteContext,
}

/// Represents the current rendering context of a markdown document.
#[derive(Debug)]
pub enum Context {
    Document,
    List(ListContext),
    BlockQuote(BlockQuoteContext),
    BlockQuoteInList(BlockQuoteInListContext),
}
